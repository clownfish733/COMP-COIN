const CHANNEL_SIZE: usize = 100;

use anyhow::Result;

use log::{info, error};

use tokio::{
    sync::{mpsc, RwLock}
};

use super::node::Node;
use super::commands::{
    MineCommand, NetworkCommand
};

use crate::{
    network::start_network_server,
    mine::start_mining_server,
    ui::start_ui_server,
};

use std::{
    sync::{
        atomic::AtomicBool,
        Arc,
    }
};

pub async fn bootstrap_node_main() -> Result<()>{
    info!("Starting Bootstrap Node");

    todo!("Implement bootstrap main")
}

pub async fn full_node_main() -> Result<()>{
    info!("Starting full Node ...");
    
    //initialising -----------------------------------------------------------------------------------

    //initiating mpsc channels
    let (miner_tx, miner_rx) = mpsc::channel::<MineCommand>(CHANNEL_SIZE);

    let (network_tx, network_rx) = mpsc::channel::<NetworkCommand>(CHANNEL_SIZE);

    let ui_save = Arc::new(AtomicBool::new(false));

    //initiiating Node
    let node = Arc::new(RwLock::new(Node::initialise()?));

    //spawning servers -----------------------------------------------------------------------------

    //spawn network server
    tokio::spawn({
        let node = Arc::clone(&node);
        let miner_tx = miner_tx.clone();
        async move {
            if let Err(e) = start_network_server(node, miner_tx, network_rx).await{
                error!("Network handling failed: {}", e);
            }
        }
    });

    //spawn UI server
    tokio::spawn({
        let node = Arc::clone(&node);
        let network_tx = network_tx.clone();
        async move{
            if let Err(e) = start_ui_server(node, network_tx, ui_save).await{
                error!("UI handling failed: {}", e);
            }
        }
    });

    //spawn mining server
    let mining_handle = tokio::spawn({
        let node = Arc::clone(&node);
        let network_tx = network_tx.clone();
        async move {
            if let Err(e) = start_mining_server(node, miner_rx, network_tx).await{
                error!("Mine handling failed: {}", e);
            }
        }
    });


    tokio::signal::ctrl_c().await?;
    println!("");
    info!("Shutting down ...");
    miner_tx.send(MineCommand::Stop).await?;
    mining_handle.await?;
    Ok(())
}

