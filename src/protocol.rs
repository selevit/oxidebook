use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOrderMessage {
    pub msg_type: String,
    pub pair: String,
    pub side: String,
    pub price: u64,
    pub volume: u64,
}
