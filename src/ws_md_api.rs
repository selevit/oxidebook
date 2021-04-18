use anyhow::{Error, Result};
use log::info;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

use crate::outbox::OutboxConsumer;
use crate::transport::create_conn_pool;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    loop {
        time::sleep(Duration::from_secs(1)).await;
    }
}

async fn run_ws_market_data_api() -> Result<(), Error> {
    info!("Running WS Market Data API");

    let pool = create_conn_pool()?;
    let consumer = OutboxConsumer::new("ws_market_data", pool.clone());

    consumer
        .subscribe(Box::new(move |envelope| {
            Box::pin(async move {
                info!("Received an envelope from outbox: {:?},", envelope);
                Ok(())
            })
        }))
        .await?;

    let addr = std::env::var("WS_MD_API_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4040".into());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }

    Ok(())
}

pub fn run() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(run_ws_market_data_api())?;
    Ok(())
}
