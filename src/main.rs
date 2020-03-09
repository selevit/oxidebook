pub mod order_book;
use order_book::{Order, OrderBook, Side};

fn main() {
    let mut order_book = OrderBook::new_with_orders(vec![
        Order::new(Side::Buy, 1000, 10),
        Order::new(Side::Buy, 1001, 5),
        Order::new(Side::Buy, 999, 5),
        Order::new(Side::Buy, 888, 5),
        Order::new(Side::Sell, 1002, 22),
        Order::new(Side::Sell, 1003, 10),
    ])
    .unwrap();
    let sell_order1 = Order::new(Side::Sell, 1003, 22);
    order_book.place(sell_order1).unwrap();

    print!("{}", order_book);
}
