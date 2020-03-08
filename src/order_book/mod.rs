//! An implementation of a trading order book.
//!
//! Provides structures and methods for matching and filling exchange orders.
use rbtree::RBTree;
use std::cmp::{min, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::error::Error;
use std::option::Option;
use std::vec::Vec;
use uuid::Uuid;

/// An error which can occur when placing an order
#[derive(Debug)]
pub enum PlacingError {
    Cancelled,
}

/// A side of the exchange order book (buy or sell)
#[derive(PartialEq, Debug, Clone, Copy, Eq, PartialOrd)]
pub enum Side {
    Buy,
    Sell,
}

/// An order key in the RBTree which is used for storing orders in the correct order.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
struct TreeKey {
    side: Side,
    price: u64,
    seq_id: u64,
}

/// Buy orders with higher price go first.
///
/// Sell orders with higher price go last.
/// If prices are equal, we order them by sequence id (placing ordering).
impl Ord for TreeKey {
    fn cmp(&self, other: &TreeKey) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.seq_id.cmp(&other.seq_id),
            cmp if self.side == Side::Sell => cmp,
            cmp => cmp.reverse(),
        }
    }
}

/// An exchange order for buying or selling assets.
///
/// All prices and volumes are present as integers in base values (e.g. Satoshi or Wei)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub price: u64,
    pub volume: u64,
}

impl Order {
    /// Creates new IoC order.
    pub fn new(side: Side, price: u64, volume: u64) -> Self {
        Order { id: Uuid::new_v4(), side, price, volume }
    }

    fn _tree_key(&self, seq_id: u64) -> TreeKey {
        TreeKey { side: self.side, price: self.price, seq_id }
    }
}

/// A deal which is the result of orders filling.
///
/// Stores the state of taker and maker orders before the deal.
#[derive(Debug, Eq, PartialEq)]
pub struct Deal {
    taker_order: Order,
    maker_order: Order,
    volume: u64,
}

/// A trading order book.
///
/// Provides the functionality for matching and filling exchange orders.
#[derive(Debug)]
pub struct OrderBook {
    next_seq_id: u64,
    buy_levels: RBTree<TreeKey, Order>,
    sell_levels: RBTree<TreeKey, Order>,
    by_uuid: HashMap<Uuid, TreeKey>,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    /// Creates new empty order book
    pub fn new() -> Self {
        OrderBook {
            next_seq_id: 0,
            buy_levels: RBTree::new(),
            sell_levels: RBTree::new(),
            by_uuid: HashMap::new(),
        }
    }

    /// Creates a new orderbook with predefined orders.
    ///
    /// Returns an error if some of passed orders can be filled.
    pub fn new_with_orders(orders: Vec<Order>) -> Result<Self, Box<dyn Error>> {
        let mut book = Self::new();

        for order in orders {
            match book.place(order) {
                Ok(deals) if !deals.is_empty() => {
                    return Err("Cannot construct the orderbook with orders which match between each other".into())
                }
                Err(e) => return Err(format!("An error occurred while placing some of the orders: {:?}", e).into()),
                Ok(_) => {}
            };
        }

        Ok(book)
    }

    /// Places the order to the order book and tries to match it with existing orders.
    ///
    /// Returns a list of deals if filling occured.
    /// Returns an error if the order cannot be placed.
    pub fn place(&mut self, order: Order) -> Result<Vec<Deal>, PlacingError> {
        let (taker_side, maker_side) = match order.side {
            Side::Buy => (&mut self.buy_levels, &mut self.sell_levels),
            Side::Sell => (&mut self.sell_levels, &mut self.buy_levels),
        };

        let mut removed_orders: Vec<(TreeKey, Order)> = Vec::new();
        let mut deals: Vec<Deal> = Vec::new();
        let mut order = order;

        for (key, maker_order) in maker_side.iter_mut() {
            match order.price.cmp(&maker_order.price) {
                Ordering::Less if order.side == Side::Buy => break,
                Ordering::Greater if order.side == Side::Sell => break,
                _ => {}
            }

            let deal_volume = min(maker_order.volume, order.volume);
            deals.push(Deal {
                taker_order: order,
                maker_order: *maker_order,
                volume: deal_volume,
            });

            maker_order.volume -= deal_volume;
            if maker_order.volume == 0 {
                removed_orders.push((*key, *maker_order));
            }

            order.volume -= deal_volume;
            if order.volume == 0 {
                break;
            }
        }

        for (key, order) in &removed_orders {
            maker_side.remove(&key);
            self.by_uuid.remove(&order.id);
        }

        if order.volume != 0 {
            let key = order._tree_key(self.next_seq_id);
            taker_side.insert(key, order);
            self.by_uuid.insert(order.id, key);
            self.next_seq_id += 1;
        }

        Ok(deals)
    }

    pub fn get_order(&self, id: Uuid) -> Option<&Order> {
        match self.by_uuid.get(&id) {
            Some(key) => {
                let tree = if key.side == Side::Sell {
                    &self.sell_levels
                } else {
                    &self.buy_levels
                };
                let order = tree.get(key).unwrap();
                Some(order)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests;
