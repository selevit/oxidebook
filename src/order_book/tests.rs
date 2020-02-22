use super::{Deal, Order, OrderBook, Side};

#[test]
fn simple_matching() {
    let maker_order = Order::new(Side::Sell, 4500, 7, 1);
    let taker_order = Order::new(Side::Buy, 4900, 20, 2);
    let expected_deal = Deal { taker_order, maker_order, volume: 7 };
    let remaining_buy = Order::new(Side::Buy, 4900, 13, 2);

    let mut book = OrderBook::new_with_orders(vec![maker_order]).unwrap();
    let deals = book.place(taker_order).unwrap();
    assert_eq!(deals.len(), 1);
    assert_eq!(deals[0], expected_deal);
    assert_eq!(book.sell_levels.len(), 0);
    assert_eq!(book.buy_levels.len(), 1);

    let buys: Vec<&Order> = book.buy_levels.values().collect();
    assert_eq!(buys[0], &remaining_buy);
}
