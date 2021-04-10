use anyhow::{Error, Result};
use log::info;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

use futures_channel::mpsc::UnboundedSender;

use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
use std::time::Duration;

use async_std::task;

async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    info!("Incoming TCP connection from: {}", addr);

    tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    loop {
        task::sleep(Duration::from_secs(1)).await;
    }
}

async fn run_ws_market_data_api() -> Result<(), Error> {
    info!("Running WS Market Data API");

    let addr = std::env::var("WS_MD_API_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4040".into());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}

pub fn run() -> Result<()> {
    let mut rt = Runtime::new()?;
    rt.block_on(run_ws_market_data_api())?;
    Ok(())
}
