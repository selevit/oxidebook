use crate::protocol;
use crate::protocol::OutboxEnvelope;
use anyhow::{Error, Result};
use deadpool_lapin::Pool;
use futures_util::stream::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use log::info;
use std::future::Future;
use std::pin::Pin;

const OUTBOX_QUEUE_NAME: &str = "outbox";

pub type OutboxHandlerResult = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
pub type OutboxHandler = Box<dyn Fn(OutboxEnvelope) -> OutboxHandlerResult>;

pub struct OutboxConsumer<'a> {
    consumer_name: &'a str,
    conn_pool: Pool,
}

impl<'a> OutboxConsumer<'a> {
    pub fn new(consumer_name: &'a str, conn_pool: Pool) -> Self {
        OutboxConsumer { consumer_name, conn_pool }
    }

    pub async fn subscribe(&self, handler: OutboxHandler) -> Result<(), Error> {
        let conn = self.conn_pool.get().await?;
        let channel = conn.create_channel().await?;
        let mut consumer = channel
            .clone()
            .basic_consume(
                OUTBOX_QUEUE_NAME,
                self.consumer_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Starting consuming outbox");

        while let Some(delivery) = consumer.next().await {
            let delivery =
                delivery.expect("error caught in the outbox consumer"); // TODO: proxy the error with ? operator
            let outbox_env: protocol::OutboxEnvelope =
                serde_json::from_slice(&delivery.data)?;
            handler(outbox_env).await?;
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?;
        }

        Ok(())
    }
}