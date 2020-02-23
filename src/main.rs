pub mod order_book;
use order_book::{Order, OrderBook, Side};

fn main() {
    let buy_order1 =
        Order { side: Side::Buy, price: 1000, volume: 10, user_id: 1 };
    let buy_order2 =
        Order { side: Side::Buy, price: 1001, volume: 5, user_id: 1 };

    let mut order_book =
        OrderBook::new_with_orders(vec![buy_order1, buy_order2]).unwrap();

    let sell_order1 =
        Order { side: Side::Sell, price: 1000, volume: 22, user_id: 2 };

    let deals = order_book.place(sell_order1).unwrap();

    dbg!(deals);
    dbg!(order_book);
}
