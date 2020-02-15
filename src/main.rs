pub mod order_book;
use order_book::{Order, OrderBook, Side};

fn main() {
    let mut order_book = OrderBook::new();

    let buy_order = Order {
        side: Side::Buy,
        price: 1000,
        volume: 10,
        seq_id: 1,
        user_id: 1,
    };
    let sell_order = Order {
        side: Side::Sell,
        price: 1000,
        volume: 10,
        seq_id: 2,
        user_id: 2,
    };

    order_book.place(buy_order).unwrap();
    let deals = order_book.place(sell_order).unwrap();

    dbg!(deals);
    dbg!(order_book);
}
