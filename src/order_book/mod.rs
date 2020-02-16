use rbtree::RBTree;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::vec::Vec;

#[derive(Debug)]
pub enum PlacingError {
    Cancelled,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, PartialOrd)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
struct TreeKey {
    side: Side,
    price: u64,
    seq_id: u64,
}

impl Ord for TreeKey {
    fn cmp(&self, other: &TreeKey) -> Ordering {
        let cmp_result = self.price.cmp(&other.price);
        match (self.side, cmp_result) {
            (Side::Buy, Ordering::Greater) => Ordering::Less,
            (Side::Buy, Ordering::Less) => Ordering::Greater,
            (_, Ordering::Equal) => self.seq_id.cmp(&other.seq_id),
            _ => cmp_result,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub side: Side,
    pub price: u64,
    pub volume: u64,
    pub user_id: u64,
}

impl Order {
    fn _tree_key(&self, seq_id: u64) -> TreeKey {
        TreeKey {
            side: self.side,
            price: self.price,
            seq_id: seq_id,
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
    next_seq_id: u64,
    buy_levels: RBTree<TreeKey, Order>,
    sell_levels: RBTree<TreeKey, Order>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            next_seq_id: 0,
            buy_levels: RBTree::new(),
            sell_levels: RBTree::new(),
        }
    }

    pub fn place(&mut self, order: Order) -> Result<Vec<Deal>, PlacingError> {
        let (taker_side, maker_side) = match order.side {
            Side::Buy => (&mut self.buy_levels, &mut self.sell_levels),
            Side::Sell => (&mut self.sell_levels, &mut self.buy_levels),
        };

        let mut order = order;
        let mut deals: Vec<Deal> = Vec::new();
        let mut removed_orders: Vec<TreeKey> = Vec::new();

        for (key, maker_order) in maker_side.iter_mut() {
            match (order.side, order.price.cmp(&maker_order.price)) {
                (Side::Sell, Ordering::Greater) | (Side::Buy, Ordering::Less) => break,
                _ => {}
            }

            let original_taker_order = order;
            let deal_volume = std::cmp::min(maker_order.volume, order.volume);
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
            taker_side.insert(order._tree_key(self.next_seq_id), order);
            self.next_seq_id += 1
        }

        for k in &removed_orders {
            maker_side.remove(&k);
        }

        Ok(deals)
    }
}
