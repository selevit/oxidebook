use std::convert::Infallible;
use tokio::runtime::Runtime;
use warp::{http::StatusCode, reject, Filter, Rejection, Reply};

use lapin::{options::QueueDeclareOptions, types::FieldTable};

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
    let mut conn = pool.get().await.unwrap();
    let consuming_channel = conn.create_channel().await.unwrap();
    let inbox_queue = consuming_channel
        .queue_declare(
            "inbox",
            QueueDeclareOptions::default(),
            FieldTable::default(),
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
