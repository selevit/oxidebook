use super::{Deal, Order, OrderBook, Side};

struct TestCase {
    initial_orders: Vec<Order>,
    placed_order: Order,
    expected_deals: Vec<Deal>,
    remaining_buys: Vec<Order>,
    remaining_sells: Vec<Order>,
}

impl TestCase {
    fn run(self) {
        let mut book = OrderBook::new_with_orders(self.initial_orders).unwrap();
        let deals = book.place(self.placed_order).unwrap();
        let buys: Vec<Order> = book.buy_levels.values().cloned().collect();
        let sells: Vec<Order> = book.sell_levels.values().cloned().collect();
        assert_eq!(deals, self.expected_deals);
        assert_eq!(buys, self.remaining_buys);
        assert_eq!(sells, self.remaining_sells);
    }
}

impl Order {
    fn buy(price: u64, volume: u64) -> Self {
        Order::new(Side::Buy, price, volume)
    }

    fn sell(price: u64, volume: u64) -> Self {
        Order::new(Side::Sell, price, volume)
    }

    fn with_volume(mut self, volume: u64) -> Self {
        self.volume = volume;
        self
    }
}

#[test]
fn place_sell_order_and_fill_it_fully() {
    let initial_orders =
        vec![Order::buy(5200, 3), Order::buy(5100, 12), Order::buy(4700, 10)];
    let placed_order = Order::sell(4800, 15);
    let expected_deals = vec![
        Deal {
            taker_order: placed_order,
            maker_order: initial_orders[0],
            volume: 3,
        },
        Deal {
            taker_order: placed_order.with_volume(12),
            maker_order: initial_orders[1],
            volume: 12,
        },
    ];
    let remaining_sells = vec![];
    let remaining_buys = vec![initial_orders[2]];

    TestCase {
        placed_order,
        initial_orders,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_sell_order_and_fill_it_partially() {
    let initial_orders =
        vec![Order::buy(5200, 3), Order::buy(5100, 11), Order::buy(4700, 10)];
    let placed_order = Order::sell(4800, 15);
    let expected_deals = vec![
        Deal {
            taker_order: placed_order,
            maker_order: initial_orders[0],
            volume: 3,
        },
        Deal {
            taker_order: placed_order.with_volume(12),
            maker_order: initial_orders[1],
            volume: 11,
        },
    ];
    let remaining_sells = vec![placed_order.with_volume(1)];
    let remaining_buys = vec![initial_orders[2]];

    TestCase {
        placed_order,
        initial_orders,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_sell_order_and_fill_it_partially_exceeding_buys() {
    let maker_order = Order::buy(5000, 9);
    let placed_order = Order::sell(4800, 10);
    let expected_deals =
        vec![Deal { taker_order: placed_order, maker_order, volume: 9 }];
    let remaining_sells = vec![placed_order.with_volume(1)];
    let remaining_buys = vec![];

    TestCase {
        initial_orders: vec![maker_order],
        placed_order,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_sell_order_without_filling() {
    let initial_buys =
        vec![Order::buy(5200, 3), Order::buy(5100, 12), Order::buy(4700, 10)];
    let initial_sells = vec![
        Order::sell(5300, 100),
        Order::sell(5350, 200),
        Order::sell(5400, 300),
    ];
    let mut initial_orders = initial_buys.clone();
    initial_orders.extend(initial_sells.iter().cloned());

    let placed_order = Order::sell(5250, 15);
    let remaining_buys = initial_buys;
    let remaining_sells = vec![
        placed_order,
        initial_sells[0],
        initial_sells[1],
        initial_sells[2],
    ];

    TestCase {
        placed_order,
        initial_orders,
        expected_deals: vec![],
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_buy_order_and_fill_it_partially_exceeding_sells() {
    let maker_order = Order::sell(4500, 7);
    let placed_order = Order::buy(4900, 20);
    let expected_deals =
        vec![Deal { taker_order: placed_order, maker_order, volume: 7 }];
    let remaining_buys = vec![placed_order.with_volume(13)];
    let remaining_sells = vec![];

    TestCase {
        initial_orders: vec![maker_order],
        placed_order,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_buy_order_and_fill_it_partially_by_better_price() {
    let initial_orders =
        vec![Order::sell(4500, 7), Order::sell(4800, 3), Order::sell(5100, 30)];
    let placed_order = Order::buy(4900, 20);
    let expected_deals = vec![
        Deal {
            taker_order: placed_order,
            maker_order: initial_orders[0],
            volume: 7,
        },
        Deal {
            taker_order: placed_order.with_volume(13),
            maker_order: initial_orders[1],
            volume: 3,
        },
    ];
    let remaining_sells = vec![initial_orders[2]];
    let remaining_buys = vec![placed_order.with_volume(10)];

    TestCase {
        placed_order,
        initial_orders,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}

#[test]
fn place_buy_order_and_fill_it_partially_by_better_price_exceeding_sells() {
    let initial_orders = vec![Order::sell(4500, 7), Order::sell(4800, 3)];
    let placed_order = Order::buy(4900, 20);
    let expected_deals = vec![
        Deal {
            taker_order: placed_order,
            maker_order: initial_orders[0],
            volume: 7,
        },
        Deal {
            taker_order: placed_order.with_volume(13),
            maker_order: initial_orders[1],
            volume: 3,
        },
    ];
    let remaining_sells = vec![];
    let remaining_buys = vec![placed_order.with_volume(10)];

    TestCase {
        placed_order,
        initial_orders,
        expected_deals,
        remaining_buys,
        remaining_sells,
    }
    .run()
}
