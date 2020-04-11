use std::convert::Infallible;
use tokio::runtime::Runtime;
use warp::Filter;

use log::info;

async fn _run() {
    info!("Running REST API server");

    async fn create_order_handler() -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::reply())
    }

    let create_order = warp::path!("api" / "v1" / "create-order")
        .and_then(create_order_handler);

    let routes = warp::get().and(create_order);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

pub fn run() {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(_run());
}
