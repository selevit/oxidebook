use crate::order_book::{Order, OrderBook, Side};
use futures_executor::LocalPool;
use futures_util::{future::FutureExt, stream::StreamExt};
use std::collections::HashMap;

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

    pub fn run(&mut self) -> lapin::Result<()> {
        let mut executor = LocalPool::new();

        executor.run_until(async {
            let addr = std::env::var("AQMP_ADDR")
                .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

            let conn =
                Connection::connect(&addr, ConnectionProperties::default())
                    .await?;

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

                    let order_book =
                        self.pairs.get_mut("BTC_USD").expect("invalid pair");

                    order_book
                        .place(Order::new(Side::Buy, 6500, 50_000_000))
                        .expect("placing error");

                    consuming_channel
                        .basic_ack(
                            delivery.delivery_tag,
                            BasicAckOptions::default(),
                        )
                        .map(|_| ())
                })
                .await;

            Ok(())
        })
    }
}
