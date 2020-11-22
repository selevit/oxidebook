use enum_dispatch::enum_dispatch;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[enum_dispatch]
pub trait MessageWithId {
    fn get_id(&self) -> Uuid;
}

#[enum_dispatch]
pub trait MessageWithCorrelationId {
    fn get_correlation_id(&self) -> Uuid;
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

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderPlaced {
    pub inbox_id: Uuid,
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
    pub order_id: Uuid,
}

impl MessageWithCorrelationId for OrderPlaced {
    fn get_correlation_id(&self) -> Uuid {
        self.inbox_id
    }
}

#[enum_dispatch(MessageWithId)]
#[derive(Deserialize, Serialize, Debug)]
pub enum InboxMessage {
    PlaceOrder(PlaceOrder),
}

#[enum_dispatch(MessageWithCorrelationId)]
#[derive(Deserialize, Serialize, Debug)]
pub enum OutboxMessage {
    OrderPlaced(OrderPlaced),
}
