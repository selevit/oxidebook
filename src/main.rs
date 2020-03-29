pub mod core;
pub mod order_book;

fn main() {
    let mut core = core::Exchange::new();
    core.add_pair("BTC_USD").unwrap();
}
