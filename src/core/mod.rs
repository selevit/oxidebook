use crate::order_book::OrderBook;
use std::collections::HashMap;

pub struct Exchange<'a> {
    pairs: HashMap<&'a str, OrderBook>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddPairError {
    AlreadyExists,
}

impl<'a> Default for Exchange<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Exchange<'a> {
    pub fn new() -> Self {
        Exchange { pairs: HashMap::new() }
    }

    pub fn add_pair(&mut self, pair_name: &'a str) -> Result<(), AddPairError> {
        if self.pairs.contains_key(pair_name) {
            return Err(AddPairError::AlreadyExists);
        }
        self.pairs.insert(pair_name, OrderBook::new());
        Ok(())
    }
}
