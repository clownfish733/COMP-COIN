use std::{
    sync::Arc,
};

use anyhow::Result;

use log::{info};

use tokio::{
    sync::{RwLock, mpsc},
};

use crate::node::{
    MineCommand,
    NetworkCommand,
    Node,
};

pub async fn start_mining_server(
    node: Arc<RwLock<Node>>,
    mut miner_rx: mpsc::Receiver<MineCommand>,
    network_tx: mpsc::Sender<NetworkCommand>,
) -> Result<()>{
    info!("Started Mining Server");
    
    while let Some(command) = miner_rx.recv().await{
        match command{
            MineCommand::Stop => {
                info!("Shutting down mining threads");
                break;
            }
        }
    };

    Ok(())
}