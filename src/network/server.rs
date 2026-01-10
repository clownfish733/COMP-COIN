const CHANNEL_SIZE: usize = 100;

use tokio::{
    sync::{RwLock, mpsc},
    net::TcpListener,
};

use std::{
    sync::Arc,
};

use anyhow::Result;

use log::{info};

use crate::node::{
    Node,
    NetworkCommand,
    MineCommand,
};

use super::connection_message::{
    ConnectionEvent,
    ConnectionResponse,
};

use super::peers::PeerManager;

pub async fn start_network_server(
    node: Arc<RwLock<Node>>,
    miner_tx: mpsc::Sender<MineCommand>,
    mut network_rx: mpsc::Receiver<NetworkCommand>,
) -> Result<()>{

    info!("Started Network Server");

    //initiating listener
    let port = node.read().await.config.get_port();
    let socket_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&socket_addr).await?;
    info!("Started listening on {}", socket_addr);


    let peer_manager = Arc::new(RwLock::new(PeerManager::new()));
    let (event_tx, event_rx) = mpsc::channel::<ConnectionEvent>(CHANNEL_SIZE);

    loop{
        let (stream, network_address) = listener.accept().await?;

        let (response_tx, response_rx) = mpsc::channel::<ConnectionResponse>(CHANNEL_SIZE);

        let (reader, writer) = stream.into_split();

    }
}