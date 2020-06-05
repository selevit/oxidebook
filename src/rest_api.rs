use std::convert::Infallible;
use tokio::runtime::Runtime;
use warp::Filter;

use lapin::{options::BasicPublishOptions, BasicProperties};

use log::info;

use deadpool_lapin::{Config, Pool};

fn with_lapin_pool(
    pool: Pool,
) -> impl Filter<Extract = (Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn create_order_handler(
    pool: Pool,
) -> Result<impl warp::Reply, Infallible> {
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel().await.unwrap();
    let payload = b"hello, world";
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
    Ok(warp::reply::reply())
}

async fn _run() {
    let cfg = Config::from_env("AMQP").unwrap();
    let pool = cfg.create_pool();
    info!("Running REST API server");
    let routes = warp::get()
        .and(with_lapin_pool(pool.clone()))
        .and_then(create_order_handler);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

pub fn run() {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(_run());
}
