use rbtree::RBTree;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::error::Error;
use std::fmt;
use std::vec::Vec;

#[derive(Debug)]
pub enum PlacingError {
    Cancelled,
}

impl fmt::Display for PlacingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order placing error")
    }
}

impl Error for PlacingError {
    fn description(&self) -> &str {
        match self {
            PlacingError::Cancelled => "The order has been cancelled",
            _ => "Unknown order placing error",
        }
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub struct OrderBook {
    buy: OrderBookSide,
    sell: OrderBookSide,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub side: Side,
    pub price: u64,
    pub volume: u64,
    pub seq_id: u64,
    pub user_id: u64,
}

impl Order {
    fn tree_key(&self) -> OrderTreeKey {
        return OrderTreeKey {
            side: self.side,
            price: self.price,
            seq_id: self.seq_id,
        };
    }
}

#[derive(Debug, Clone, Copy)]
struct OrderTreeKey {
    side: Side,
    price: u64,
    seq_id: u64,
}

impl Ord for OrderTreeKey {
    fn cmp(&self, other: &OrderTreeKey) -> Ordering {
        return match self.side {
            Side::Buy => {
                if self.price < other.price {
                    Ordering::Greater
                } else if self.price > other.price {
                    Ordering::Less
                } else {
                    self.seq_id.cmp(&other.seq_id)
                }
            }
            Side::Sell => {
                if self.price < other.price {
                    Ordering::Less
                } else if self.price > other.price {
                    Ordering::Greater
                } else {
                    self.seq_id.cmp(&other.seq_id)
                }
            }
        };
    }
}

impl Eq for OrderTreeKey {}

impl PartialEq for OrderTreeKey {
    fn eq(&self, other: &OrderTreeKey) -> bool {
        self.side == other.side && self.price == other.price && self.seq_id == other.seq_id
    }
}

impl PartialOrd for OrderTreeKey {
    fn partial_cmp(&self, other: &OrderTreeKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Deal {
    taker_order: Order,
    maker_order: Order,
    volume: u64,
}

impl Deal {
    fn new(taker_order: Order, maker_order: Order, volume: u64) -> Deal {
        Deal {
            taker_order: taker_order,
            maker_order: maker_order,
            volume: volume,
        }
    }
}

#[derive(Debug)]
struct OrderBookSide {
    orders: RBTree<OrderTreeKey, Order>,
}

impl OrderBookSide {
    fn new() -> OrderBookSide {
        let orders = RBTree::new();
        OrderBookSide { orders: orders }
    }
}

impl OrderBook {
    pub fn new() -> OrderBook {
        let buy = OrderBookSide::new();
        let sell = OrderBookSide::new();
        let orderbook = OrderBook {
            buy: buy,
            sell: sell,
        };
        orderbook
    }

    pub fn place(&mut self, order: Order) -> Result<Vec<Deal>, PlacingError> {
        let (taker_side, maker_side) = match order.side {
            Side::Buy => (&mut self.buy, &mut self.sell),
            Side::Sell => (&mut self.sell, &mut self.buy),
            _ => unreachable!(),
        };

        let mut deals: Vec<Deal> = Vec::new();
        let mut order = order;
        let mut removed_orders: Vec<OrderTreeKey> = Vec::new();

        for (key, maker_order) in maker_side.orders.iter_mut() {
            if order.side == Side::Buy && order.price > maker_order.price {
                break;
            }
            if order.side == Side::Buy && order.price < maker_order.price {
                break;
            }

            let deal_volume = if maker_order.volume < order.volume {
                maker_order.volume
            } else {
                order.volume
            };

            let original_taker_order = order;
            deals.push(Deal::new(original_taker_order, *maker_order, deal_volume));

            order.volume -= deal_volume;
            maker_order.volume -= deal_volume;

            if maker_order.volume == 0 {
                removed_orders.push(*key);
            }
            if order.volume == 0 {
                break;
            }
        }

        if order.volume > 0 {
            taker_side.orders.insert(order.tree_key(), order);
        }

        for k in &removed_orders {
            maker_side.orders.remove(&k);
        }

        Ok(deals)
    }
}
