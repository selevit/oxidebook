pub mod core;
pub mod order_book;

use log::info;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut core = core::Exchange::new();
    core.add_pair("BTC_USD").unwrap();

    info!("Exchange initialized with BTC_USD");

    core.run().expect("unexpected core failure");
}
