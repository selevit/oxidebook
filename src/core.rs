use crate::order_book::{Order, OrderBook, Side};
use crate::protocol::{
    self, InboxMessage, MessageWithId, OutboxEnvelope, OutboxMessage,
};
use anyhow::{Context, Result};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use thiserror::Error;
use tokio::runtime::Runtime;

use amq_protocol_types::ShortString;
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

#[derive(Error, Debug)]
pub enum AddPairError {
    #[error("trading pair already exists")]
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

    pub async fn run(&mut self) -> Result<()> {
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
            let inbox_message: InboxMessage =
                serde_json::from_slice(&delivery.data)?;
            let inbox_id = inbox_message.get_id();
            let mut outbox = OutboxEnvelope::new(inbox_id);

            match inbox_message {
                InboxMessage::PlaceOrder(message) => {
                    info!("Place order message: {:?}", message);
                    let order_book = self
                        .pairs
                        .get_mut(message.pair.as_str())
                        .context("invalid pair")?;

                    // TODO: serialize enums directly
                    let side = if message.side == "buy" {
                        Side::Buy
                    } else {
                        Side::Sell
                    };
                    let order = Order::new(side, message.price, message.volume);

                    let deals = order_book.place(order)?;

                    info!("New order placed");
                    info!("{}", order_book);

                    outbox.add_message(OutboxMessage::OrderPlaced(
                        protocol::OrderPlaced {
                            order_id: order.id,
                            side: message.side,
                            price: order.price,
                            volume: order.volume,
                            pair: message.pair,
                        },
                    ));

                    for deal in deals {
                        outbox.add_message(OutboxMessage::OrderFilled(
                            protocol::OrderFilled {
                                maker_order: deal.maker_order,
                                taker_order: deal.taker_order,
                                volume: deal.volume,
                            },
                        ));
                    }
                }
                InboxMessage::CancelOrder(message) => {
                    info!("Cancel order message: {:?}", message);
                    let order_book = self
                        .pairs
                        .get_mut(message.pair.as_str())
                        .context("invalid pair")?;

                    outbox.add_message(
                        match order_book.cancel_order(message.order_id) {
                            Ok(_) => OutboxMessage::OrderCancelled(
                                protocol::OrderCancelled {
                                    pair: message.pair,
                                    order_id: message.order_id,
                                },
                            ),
                            Err(_) => OutboxMessage::OrderNotFound(
                                protocol::OrderNotFound {
                                    pair: message.pair,
                                    order_id: message.order_id,
                                },
                            ),
                        },
                    );
                }
            };

            let outbox_payload = serde_json::to_vec(&outbox)?;
            let correlation_id = outbox.inbox_correlation_id;

            producing_channel
                .basic_publish(
                    "",
                    outbox_queue.name().as_str(),
                    BasicPublishOptions::default(),
                    outbox_payload,
                    BasicProperties::default().with_correlation_id(
                        ShortString::from(
                            correlation_id.to_hyphenated().to_string(),
                        ),
                    ),
                )
                .await?;

            // FIXME: orders's sorting with the same price seems to be working incorrectly (tested with sells). Grasp and fix.
            consuming_channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?;
        }

        Ok(())
    }
}

pub fn run() -> Result<()> {
    let mut exchange = Exchange::new();
    exchange.add_pair("BTC_USD")?;
    info!("Exchange initialized with BTC_USD");
    let rt = Runtime::new()?;
    rt.block_on(exchange.run())?;
    Ok(())
}
