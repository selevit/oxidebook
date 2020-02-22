use super::{Deal, Order, OrderBook, Side};

struct TestCase {
    existing_orders: Vec<Order>,
    placed_order: Order,
    expected_deals: Vec<Deal>,
    remaining_buys: Vec<Order>,
    remaining_sells: Vec<Order>,
}

impl TestCase {
    fn run(self) {
        let mut book = OrderBook::new_with_orders(self.existing_orders).unwrap();
        let deals = book.place(self.placed_order).unwrap();
        let buys: Vec<Order> = book.buy_levels.values().cloned().collect();
        let sells: Vec<Order> = book.sell_levels.values().cloned().collect();
        assert_eq!(deals, self.expected_deals);
        assert_eq!(buys, self.remaining_buys);
        assert_eq!(sells, self.remaining_sells);
    }
}

#[test]
fn place_buy_order_and_fill_it_partially() {
    let maker_order = Order::new(Side::Sell, 4500, 7, 1);
    let taker_order = Order::new(Side::Buy, 4900, 20, 2);
    let expected_deals = vec![Deal { taker_order, maker_order, volume: 7 }];
    let remaining_buys = vec![Order::new(Side::Buy, 4900, 13, 2)];
    let remaining_sells = vec![];

    TestCase {
        existing_orders: vec![maker_order],
        placed_order: taker_order,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_sell_order_and_fill_it_partially() {
    let maker_order = Order::new(Side::Buy, 5000, 9, 1);
    let taker_order = Order::new(Side::Sell, 4800, 10, 2);
    let expected_deals = vec![Deal { taker_order, maker_order, volume: 9 }];
    let remaining_sells = vec![Order::new(Side::Sell, 4800, 1, 2)];
    let remaining_buys = vec![];

    TestCase {
        existing_orders: vec![maker_order],
        placed_order: taker_order,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}
