use crate::order_book::{Order, OrderBook, Side};
use crate::protocol;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use tokio::runtime::Runtime;

use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use log::info;

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

    pub async fn run(&mut self) -> lapin::Result<()> {
        let addr = std::env::var("AQMP_ADDR")
            .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

        let conn =
            Connection::connect(&addr, ConnectionProperties::default()).await?;

        info!("Connected to RabbitMQ");
        let consuming_channel = conn.create_channel().await?;
        let producing_channel = conn.create_channel().await?;

        let inbox_queue = consuming_channel
            .queue_declare(
                "inbox",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let mut consumer = consuming_channel
            .clone()
            .basic_consume(
                inbox_queue.name().as_str(),
                "core",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let outbox_queue = producing_channel
            .queue_declare(
                "outbox",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Starting consuming inbox");

        while let Some(delivery) = consumer.next().await {
            let delivery =
                delivery.expect("error caught in the inbox consumer");
            let message: protocol::CreateOrderMessage =
                serde_json::from_slice(&delivery.data).unwrap();

            info!("Message: {:?}", message);
            let order_book = self
                .pairs
                .get_mut(message.pair.as_str())
                .expect("invalid pair");

            // TODO: serialize enums directly
            let side =
                if message.side == "buy" { Side::Buy } else { Side::Sell };
            let order = Order::new(side, message.price, message.volume);

            order_book.place(order).expect("placing error");

            let order_placed_msg = protocol::OrderPlacedMessage {
                msg_type: "OrderPlaced".to_owned(),
                order_id: order.id.to_hyphenated().to_string(),
                side: message.side,
                price: order.price,
                volume: order.volume,
                pair: message.pair,
            };

            let outbox_payload = serde_json::to_vec(&order_placed_msg).unwrap();

            producing_channel
                .basic_publish(
                    "",
                    outbox_queue.name().as_str(),
                    BasicPublishOptions::default(),
                    outbox_payload,
                    BasicProperties::default(),
                )
                .await
                .unwrap();

            // FIXME: orders's sorting with the same price seems to be working incorrectly (tested with sells). Grasp and fix.

            info!("New order placed");
            info!("{}", order_book);
            consuming_channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
                .unwrap();
        }

        Ok(())
    }
}

pub fn run() {
    let mut exchange = Exchange::new();
    exchange.add_pair("BTC_USD").unwrap();
    info!("Exchange initialized with BTC_USD");
    let mut rt = Runtime::new().unwrap();
    rt.block_on(exchange.run()).unwrap();
}
