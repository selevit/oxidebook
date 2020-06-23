use crate::protocol;
use futures::join;
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::runtime::Runtime;
use warp::Filter;

use futures_util::stream::StreamExt;
use lapin::types::FieldTable;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions},
    BasicProperties,
};

use log::info;

use deadpool_lapin::{Config, Pool};

fn with_lapin_pool(
    pool: Pool,
) -> impl Filter<Extract = (Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

#[derive(Deserialize, Serialize)]
struct CreateOrderRequest {
    pair: String,
    side: String,
    // TODO:These values should be decimal strings at this abstraction level
    price: u64,
    volume: u64,
}

#[derive(Deserialize, Serialize)]
struct CreateOrderResponse {
    order_id: String,
}

async fn create_order_handler(
    pool: Pool,
    req: CreateOrderRequest,
) -> Result<impl warp::Reply, Infallible> {
    // TODO: validate request
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel().await.unwrap();
    let message = protocol::CreateOrderMessage {
        msg_type: "CreateOrderMessage".to_owned(),
        price: req.price,
        side: req.side,
        pair: req.pair,
        volume: req.volume,
    };
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
    let response = CreateOrderResponse { order_id: "fake".into() };
    Ok(warp::reply::json(&response))
}

async fn run_outbox_consumer(pool: Pool) {
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
        let message: protocol::OrderPlacedMessage =
            serde_json::from_slice(&delivery.data).unwrap();
        info!("Received a message from outbox: {:?}", &message);
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await
            .unwrap();
    }
}

async fn _run() {
    let cfg = Config::from_env("AMQP").unwrap();
    let pool = cfg.create_pool();
    info!("Running REST API server");

    let routes = warp::post()
        .and(warp::path("create-order"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(with_lapin_pool(pool.clone()))
        .and(warp::body::json())
        .and_then(create_order_handler);

    let server_fut = warp::serve(routes).run(([127, 0, 0, 1], 3030));
    let outbox_consumer_fut = run_outbox_consumer(pool);

    join!(server_fut, outbox_consumer_fut);
}

pub fn run() {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(_run());
}
