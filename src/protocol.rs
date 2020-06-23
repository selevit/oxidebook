use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOrderMessage {
    pub msg_id: Uuid,
    pub msg_type: String,
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderPlacedMessage {
    pub msg_type: String,
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
    pub order_id: String,
}
