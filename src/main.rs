pub mod order_book;
use order_book::{Order, OrderBook, Side};

fn main() {
    let buy_order1 = Order::new(Side::Buy, 1000, 10);
    let buy_order2 = Order::new(Side::Buy, 1001, 5);

    let mut order_book =
        OrderBook::new_with_orders(vec![buy_order1, buy_order2]).unwrap();

    let sell_order1 = Order::new(Side::Sell, 1000, 22);
    let deals = order_book.place(sell_order1).unwrap();

    dbg!(deals);
    dbg!(order_book);
}
