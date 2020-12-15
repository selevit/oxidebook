use crate::order_book::Order;
use enum_dispatch::enum_dispatch;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[enum_dispatch]
pub trait MessageWithId {
    fn get_id(&self) -> Uuid;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlaceOrder {
    pub msg_id: Uuid,
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
}

impl MessageWithId for PlaceOrder {
    fn get_id(&self) -> Uuid {
        self.msg_id
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CancelOrder {
    pub msg_id: Uuid,
    pub pair: String,
    pub order_id: Uuid,
}

impl MessageWithId for CancelOrder {
    fn get_id(&self) -> Uuid {
        self.msg_id
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderPlaced {
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
    pub order_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderFilled {
    pub taker_order: Order,
    pub maker_order: Order,
    pub volume: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderCancelled {
    pub order_id: Uuid,
    pub pair: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderNotFound {
    pub order_id: Uuid,
    pub pair: String,
}

#[enum_dispatch(MessageWithId)]
#[derive(Deserialize, Serialize, Debug)]
pub enum InboxMessage {
    PlaceOrder(PlaceOrder),
    CancelOrder(CancelOrder),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OutboxMessage {
    OrderPlaced(OrderPlaced),
    OrderFilled(OrderFilled),
    OrderCancelled(OrderCancelled),
    OrderNotFound(OrderNotFound),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OutboxEnvelope {
    pub inbox_correlation_id: Uuid,
    pub messages: Vec<OutboxMessage>,
}

impl OutboxEnvelope {
    pub fn new(inbox_correlation_id: Uuid) -> Self {
        OutboxEnvelope { inbox_correlation_id, messages: vec![] }
    }

    pub fn add_message(&mut self, msg: OutboxMessage) {
        self.messages.push(msg);
    }
}
