const CHANNEL_SIZE: usize = 100;

use anyhow::{Result, anyhow};

use clap::Parser;
use log::{error, info, warn};

use tokio::{
    sync::{RwLock, mpsc}, time
};

use super::node::Node;
use super::commands::{
    MineCommand, NetworkCommand,
};
use super::parser::Cli;

use crate::{
    network::start_network_server,
    mine::start_mining_server,
    ui::start_ui_server,
};

use std::{sync::{
        Arc, atomic::{AtomicBool, Ordering}
    }, time::Duration};

const BOOTSTRAP_PORT: usize = 8333;
const BOOTSTRAP_ADDR: &str = "192.168.1.152";

async fn bootstrap_node_main(node: Arc<RwLock<Node>>) -> Result<()>{
    info!("Starting Bootstrap Node");

     //initialising -----------------------------------------------------------------------------------

    //initiating mpsc channels
    let (miner_tx, miner_rx) = mpsc::channel::<MineCommand>(CHANNEL_SIZE);

    let (network_tx, network_rx) = mpsc::channel::<NetworkCommand>(CHANNEL_SIZE);

    //spawning servers -----------------------------------------------------------------------------

    //spawn network server
    tokio::spawn({
        let node = Arc::clone(&node);
        let miner_tx = miner_tx.clone();
        let nework_tx = network_tx.clone();

        async move {
            if let Err(e) = start_network_server(
                node, 
                miner_tx, 
                network_rx, 
                nework_tx
            ).await{
                error!("Network handling failed: {}", e);
            }
        }
    });

    //spawn mining server
    let mining_handle = tokio::spawn({
        let node = Arc::clone(&node);
        let network_tx = network_tx.clone();
        async move {
            if let Err(e) = start_mining_server(
                node, 
                miner_rx, 
                network_tx).await{
                error!("Mine handling failed: {}", e);
            }
        }
    });


    tokio::signal::ctrl_c().await?;
    println!("");
    info!("Shutting down ...");
    miner_tx.send(MineCommand::Stop).await?;
    mining_handle.await?;
    node.read().await.save().await?;
    time::sleep(Duration::from_secs(1)).await;
    Ok(())
}



async fn full_node_main(node: Arc<RwLock<Node>>) -> Result<()>{
    info!("Starting full Node ...");
    
    //initialising -----------------------------------------------------------------------------------

    //initiating mpsc channels
    let (miner_tx, miner_rx) = mpsc::channel::<MineCommand>(CHANNEL_SIZE);

    let (network_tx, network_rx) = mpsc::channel::<NetworkCommand>(CHANNEL_SIZE);

    let ui_save = Arc::new(AtomicBool::new(false));

    //spawning servers -----------------------------------------------------------------------------

    //spawn network server
    tokio::spawn({
        let node = Arc::clone(&node);
        let miner_tx = miner_tx.clone();
        let nework_tx_clone = network_tx.clone();

        async move {
            if let Err(e) = start_network_server(
                node, 
                miner_tx, 
                network_rx, 
                nework_tx_clone
            ).await{
                error!("Network handling failed: {}", e);
            }
        }
    });

    //spawn UI server
    tokio::spawn({
        let node = Arc::clone(&node);
        let network_tx = network_tx.clone();
        let ui_save = Arc::clone(&ui_save);
        async move{
            if let Err(e) = start_ui_server(
                node, 
                network_tx, 
                ui_save
            ).await{
                error!("UI handling failed: {}", e);
            }
        }
    });

    //spawn mining server
    let mining_handle = tokio::spawn({
        let node = Arc::clone(&node);
        let network_tx = network_tx.clone();
        async move {
            if let Err(e) = start_mining_server(
                node, 
                miner_rx, 
                network_tx).await{
                error!("Mine handling failed: {}", e);
            }
        }
    });

    let network_tx_clone = network_tx.clone();
    if let Err(e) = network_tx_clone.send(NetworkCommand::Connect(format!("{}:{}", BOOTSTRAP_ADDR, BOOTSTRAP_PORT).parse()?)).await{
        warn!("unable to send on network channel: {}",e);
    }   
    


    tokio::signal::ctrl_c().await?;
    println!("");
    info!("Shutting down ...");
    miner_tx.send(MineCommand::Stop).await?;
    mining_handle.await?;
    ui_save.store(true, Ordering::SeqCst);
    node.read().await.save().await?;
    time::sleep(Duration::from_secs(1)).await;
    Ok(())
}

pub async fn start_server() -> Result<()>{
    let args = Cli::parse();

    let mut node = match args.operation.as_str(){
            "load" => Node::load().await?,
            "new" => Node::new().await,
            _ => {return Err(anyhow!("Error: Unknown node type '{}'. Use 'bootstrap' or 'full-node'", args.operation))}
        };

    node.set_port(args.port);
    let node = Arc::new(RwLock::new(node));
    
    match args.node_path.as_str(){
        "bootstrap" => bootstrap_node_main(node).await?,
        "full-node" => full_node_main(node).await?,
        _ => {
            return Err(anyhow!("Error: Unknown node type '{}'. Use 'bootstrap' or 'full-node'", args.node_path))
        }
    }
    Ok(())
}