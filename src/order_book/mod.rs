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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
struct TreeKey {
    side: Side,
    price: u64,
    seq_id: u64,
}

impl Ord for TreeKey {
    fn cmp(&self, other: &TreeKey) -> Ordering {
        match self.side {
            Side::Buy => {
                match self.price.cmp(&other.price) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                    _ => self.seq_id.cmp(&other.seq_id),
                }
            }
            Side::Sell => {
                match self.price.cmp(&other.price) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    _ => self.seq_id.cmp(&other.seq_id),
                }
            }
        }
    }
}

impl Eq for TreeKey {}

impl PartialEq for TreeKey {
    fn eq(&self, other: &TreeKey) -> bool {
        self.side == other.side && self.price == other.price && self.seq_id == other.seq_id
    }
}

impl PartialOrd for TreeKey {
    fn partial_cmp(&self, other: &TreeKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
    fn tree_key(&self) -> TreeKey {
        TreeKey {
            side: self.side,
            price: self.price,
            seq_id: self.seq_id,
        }
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
pub struct OrderBook {
    buy_levels: RBTree<TreeKey, Order>,
    sell_levels: RBTree<TreeKey, Order>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            buy_levels: RBTree::new(),
            sell_levels: RBTree::new(),
        }
    }

    pub fn place(&mut self, order: Order) -> Result<Vec<Deal>, PlacingError> {
        let (taker_side, maker_side) = match order.side {
            Side::Buy => (&mut self.buy_levels, &mut self.sell_levels),
            Side::Sell => (&mut self.sell_levels, &mut self.buy_levels),
            _ => unreachable!(),
        };

        let mut order = order;
        let mut deals: Vec<Deal> = Vec::new();
        let mut removed_orders: Vec<TreeKey> = Vec::new();

        for (key, maker_order) in maker_side.iter_mut() {
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
            taker_side.insert(order.tree_key(), order);
        }

        for k in &removed_orders {
            maker_side.remove(&k);
        }

        Ok(deals)
    }
}
