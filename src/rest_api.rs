extern crate futures;
extern crate tokio;
use crate::order_book::Deal;
use crate::protocol;
use crate::protocol::OutboxEnvelope;
use anyhow::{Error, Result};
use futures::join;
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::Filter;

use futures_util::stream::StreamExt;
use lapin::types::FieldTable;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions},
    BasicProperties,
};
use std::collections::HashMap;
use std::option::Option;

use log::info;

use deadpool_lapin::{Config, Pool};

fn with_lapin_pool(
    pool: Pool,
) -> impl Filter<Extract = (Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

struct OutboxResults {
    senders: Mutex<HashMap<Uuid, RefCell<Option<Sender<OutboxEnvelope>>>>>,
}

impl OutboxResults {
    pub fn new() -> Self {
        OutboxResults { senders: Mutex::new(HashMap::new()) }
    }

    pub async fn has_id(&self, uuid: Uuid) -> bool {
        // TODO: use rlock here
        self.senders.lock().await.contains_key(&uuid)
    }

    pub async fn wait_for_result(&self, uuid: Uuid) -> OutboxEnvelope {
        let (sender, receiver) = oneshot::channel::<OutboxEnvelope>();
        self.senders.lock().await.insert(uuid, RefCell::new(Some(sender)));
        receiver.await.unwrap()
    }

    pub async fn send_result(&self, uuid: Uuid, result: OutboxEnvelope) {
        if let Some(tx) =
            self.senders.lock().await.get(&uuid).unwrap().borrow_mut().take()
        {
            tx.send(result).unwrap();
        }
    }
}

fn with_outbox_results(
    outbox_results: Arc<OutboxResults>,
) -> impl Filter<Extract = (Arc<OutboxResults>,), Error = std::convert::Infallible>
       + Clone {
    warp::any().map(move || outbox_results.clone())
}

#[derive(Deserialize, Serialize)]
struct PlaceOrderRequest {
    pair: String,
    side: String,
    // TODO:These values should be decimal strings at this abstraction level
    price: u64,
    volume: u64,
}

#[derive(Deserialize, Serialize)]
struct PlaceOrderResponse {
    order_id: Uuid,
    deals: Vec<Deal>,
}

impl PlaceOrderResponse {
    fn dummy() -> Self {
        PlaceOrderResponse { order_id: Uuid::nil(), deals: vec![] }
    }
}

async fn place_order_handler(
    pool: Pool,
    outbox_results: Arc<OutboxResults>,
    req: PlaceOrderRequest,
) -> Result<impl warp::Reply, Infallible> {
    // TODO: validate request
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel().await.unwrap();
    let msg_id = Uuid::new_v4();
    let message = protocol::InboxMessage::PlaceOrder(protocol::PlaceOrder {
        msg_id,
        price: req.price,
        side: req.side,
        pair: req.pair,
        volume: req.volume,
    });
    let payload = serde_json::to_vec(&message).unwrap();

    channel
        .basic_publish(
            "",
            "inbox",
            BasicPublishOptions::default(),
            payload.to_vec(),
            BasicProperties::default(),
        )
        .await
        .unwrap();

    let outbox_envelope = outbox_results.wait_for_result(msg_id).await;
    let mut response = PlaceOrderResponse::dummy();

    for outbox_message in outbox_envelope.messages {
        match outbox_message {
            protocol::OutboxMessage::OrderPlaced(m) => {
                response.order_id = m.order_id;
            }
            protocol::OutboxMessage::OrderFilled(m) => {
                response.deals.push(Deal {
                    taker_order: m.taker_order,
                    maker_order: m.maker_order,
                    volume: m.volume,
                })
            }
            _ => unreachable!(),
        }
    }

    Ok(warp::reply::json(&response))
}

#[derive(Deserialize, Serialize)]
struct CancelOrderRequest {
    pair: String,
    order_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub enum CancelOrderResponseStatus {
    OrderCancelled,
    OrderNotFound,
}

#[derive(Deserialize, Serialize)]
struct CancelOrderResponse {
    status: CancelOrderResponseStatus,
}

async fn cancel_order_handler(
    pool: Pool,
    outbox_results: Arc<OutboxResults>,
    req: CancelOrderRequest,
) -> Result<impl warp::Reply, Infallible> {
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel().await.unwrap();
    let msg_id = Uuid::new_v4();
    let message = protocol::InboxMessage::CancelOrder(protocol::CancelOrder {
        msg_id,
        pair: req.pair,
        order_id: req.order_id,
    });
    let payload = serde_json::to_vec(&message).unwrap();
    channel
        .basic_publish(
            "",
            "inbox",
            BasicPublishOptions::default(),
            payload.to_vec(),
            BasicProperties::default(),
        )
        .await
        .unwrap();
    let outbox_envelope = outbox_results.wait_for_result(msg_id).await;
    let outbox_msg = &outbox_envelope.messages[0];

    let cancel_order_status = match outbox_msg {
        protocol::OutboxMessage::OrderCancelled(_) => {
            CancelOrderResponseStatus::OrderCancelled
        }
        protocol::OutboxMessage::OrderNotFound(_) => {
            CancelOrderResponseStatus::OrderNotFound
        }
        _ => unreachable!(),
    };

    Ok(warp::reply::json(&CancelOrderResponse { status: cancel_order_status }))
}

async fn run_outbox_consumer(
    pool: Pool,
    outbox_results: Arc<OutboxResults>,
) -> Result<()> {
    let conn = pool.get().await?;
    let channel = conn.create_channel().await?;

    let mut consumer = channel
        .clone()
        .basic_consume(
            "outbox",
            "rest_api",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Starting consuming outbox");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error caught in the outbox consumer");
        let outbox_env: protocol::OutboxEnvelope =
            serde_json::from_slice(&delivery.data)?;
        info!("Received an envelope from outbox: {:?},", &outbox_env);

        let correlation_id =
            delivery.properties.correlation_id().as_ref().unwrap().as_str();
        let msg_id = Uuid::from_str(correlation_id)?;

        info!("Correlation id: {}", msg_id);

        // TODO: think about proper routing with many API consumers
        if outbox_results.has_id(msg_id).await {
            outbox_results.send_result(msg_id, outbox_env).await;

            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?;
        }
    }

    Ok(())
}

async fn _run() -> Result<(), Error> {
    let cfg = Config::from_env("AMQP")?;
    let pool = cfg.create_pool();
    let r = Arc::new(OutboxResults::new());

    info!("Running REST API server");

    let place_order = warp::post()
        .and(warp::path("place-order"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(with_lapin_pool(pool.clone()))
        .and(with_outbox_results(r.clone()))
        .and(warp::body::json())
        .and_then(place_order_handler);

    let cancel_order = warp::post()
        .and(warp::path("cancel-order"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(with_lapin_pool(pool.clone()))
        .and(with_outbox_results(r.clone()))
        .and(warp::body::json())
        .and_then(cancel_order_handler);

    let routes = place_order.or(cancel_order);

    let server_fut = warp::serve(routes).run(([127, 0, 0, 1], 3030));
    let outbox_consumer_fut = run_outbox_consumer(pool, r.clone());
    let (consumer_result, _) = join!(outbox_consumer_fut, server_fut);
    if let Err(e) = consumer_result {
        panic!("{}", e)
    }
    Ok(())
}

pub fn run() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(_run())?;
    Ok(())
}
