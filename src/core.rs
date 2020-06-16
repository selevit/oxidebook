use crate::order_book::{Order, OrderBook, Side};
use crate::protocol;
use futures_util::{future::FutureExt, stream::StreamExt};
use std::collections::HashMap;
use tokio::runtime::Runtime;

use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
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
        let inbox_queue = consuming_channel
            .queue_declare(
                "inbox",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let consumer = consuming_channel
            .clone()
            .basic_consume(
                inbox_queue.name().as_str(),
                "core",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Starting consuming inbox");

        consumer
            .for_each(move |delivery| {
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
                order_book
                    .place(Order::new(side, message.price, message.volume))
                    .expect("placing error");

                // FIXME: orders's sorting with the same price seems to be working`` incorrectly (tested with sells). Grasp and fix.

                info!("New order placed");
                info!("{}", order_book);
                consuming_channel
                    .basic_ack(
                        delivery.delivery_tag,
                        BasicAckOptions::default(),
                    )
                    .map(|_| ())
            })
            .await;

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
