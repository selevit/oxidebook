use super::{Deal, Order, OrderBook, Side};

#[test]
fn simple_matching() {
    let mut order_book = OrderBook::new();
    let buy_order1 = Order { side: Side::Buy, price: 1000, volume: 10, user_id: 1 };
    assert_eq!(order_book.place(buy_order1).unwrap().len(), 0);

    let buy_order2 = Order { side: Side::Buy, price: 1001, volume: 5, user_id: 1 };
    assert_eq!(order_book.place(buy_order2).unwrap().len(), 0);

    let sell_order1 = Order { side: Side::Sell, price: 1000, volume: 22, user_id: 2 };
    let deals = order_book.place(sell_order1).unwrap();

    assert_eq!(deals.len(), 2);

    assert_eq!(
        deals[0],
        Deal {
            taker_order: Order { side: Side::Sell, price: 1000, volume: 22, user_id: 2 },
            maker_order: Order { side: Side::Buy, price: 1001, volume: 5, user_id: 1 },
            volume: 5,
        }
    );

    assert_eq!(
        deals[1],
        Deal {
            taker_order: Order { side: Side::Sell, price: 1000, volume: 17, user_id: 2 },
            maker_order: Order { side: Side::Buy, price: 1000, volume: 10, user_id: 1 },
            volume: 10,
        }
    );

    assert_eq!(order_book.buy_levels.len(), 0);
    assert_eq!(order_book.sell_levels.len(), 1);
}
