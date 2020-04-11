use warp::Filter;

use log::info;

pub async fn run() {
    info!("Running REST API server");
    let create_order = warp::path!("api" / "v1" / "create-order").map(|| "Ok");
    let routes = warp::get().and(create_order);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
