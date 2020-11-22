extern crate futures;
extern crate tokio;
use crate::protocol;
use crate::protocol::OutboxMessage;
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
    senders: Mutex<HashMap<Uuid, RefCell<Option<Sender<OutboxMessage>>>>>,
}

impl OutboxResults {
    pub fn new() -> Self {
        OutboxResults { senders: Mutex::new(HashMap::new()) }
    }

    pub async fn has_id(&self, uuid: Uuid) -> bool {
        // TODO: use rlock here
        self.senders.lock().await.contains_key(&uuid)
    }

    pub async fn wait_for_result(&self, uuid: Uuid) -> OutboxMessage {
        let (sender, receiver) = oneshot::channel::<OutboxMessage>();
        self.senders.lock().await.insert(uuid, RefCell::new(Some(sender)));
        receiver.await.unwrap()
    }

    pub async fn send_result(&self, uuid: Uuid, result: OutboxMessage) {
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
    let outbox_message = outbox_results.wait_for_result(msg_id).await;
    match outbox_message {
        protocol::OutboxMessage::OrderPlaced(m) => {
            let response = serde_json::to_vec(&PlaceOrderResponse {
                order_id: m.order_id,
            })
            .unwrap();
            Ok(response)
        }
    }
}

async fn run_outbox_consumer(pool: Pool, outbox_results: Arc<OutboxResults>) {
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel
        .clone()
        .basic_consume(
            "outbox",
            "rest_api",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!("Starting consuming outbox");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error caught in the outbox consumer");
        let outbox_message: protocol::OutboxMessage =
            serde_json::from_slice(&delivery.data).unwrap();
        info!("Received a message from outbox: {:?},", &outbox_message);

        let correlation_id =
            delivery.properties.correlation_id().as_ref().unwrap().as_str();
        let msg_id = Uuid::from_str(correlation_id).unwrap();

        info!("Correlation id: {}", msg_id);

        if outbox_results.has_id(msg_id).await {
            outbox_results.send_result(msg_id, outbox_message).await;

            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
                .unwrap();
        }
    }
}

async fn _run() {
    let cfg = Config::from_env("AMQP").unwrap();
    let pool = cfg.create_pool();
    let r = Arc::new(OutboxResults::new());

    info!("Running REST API server");

    let routes = warp::post()
        .and(warp::path("create-order"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(with_lapin_pool(pool.clone()))
        .and(with_outbox_results(r.clone()))
        .and(warp::body::json())
        .and_then(place_order_handler);

    let server_fut = warp::serve(routes).run(([127, 0, 0, 1], 3030));
    let outbox_consumer_fut = run_outbox_consumer(pool, r.clone());

    join!(server_fut, outbox_consumer_fut);
}

pub fn run() {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(_run());
}
