use std::sync::Arc;

use anyhow::Result;
use log::info;
use tokio::sync::{RwLock, mpsc};

const CHANNEL_SIZE: usize = 100;

use crate::{
    block::Block,
    mempool::Mempool,
    utxos::UTXOS,
    wallet::Wallet,
    network::NetworkCommand,
    mine::MineCommand,
};

pub struct Node{
    height: usize,
    mempool: Mempool,
    block_chain: Vec<Block>,
    config: Config,
    utxos: UTXOS,
    wallet: Wallet,
}

impl Node{
    fn new() -> Self{
        Self {
            height: 0, 
            mempool: Mempool::new(), 
            block_chain: Vec::new(), 
            config:Config::tmp_load(), 
            utxos: UTXOS::new(), 
            wallet: Wallet::new() 
        }
    }

    fn tmp_load() -> Result<Self>{
        Ok(Self::new())
    }
}

pub async fn bootstrap_node_main(load: bool) -> Result<()>{
    info!("Starting Bootstrap Node ...");

    //initiating mpsc channels
    
    let (miner_tx, miner_rx) = mpsc::channel::<MineCommand>(CHANNEL_SIZE);

    let (network_tx, network_rx) = mpsc::channel::<NetworkCommand>(CHANNEL_SIZE);

    let node = Arc::new(RwLock::new(
        match load {
            true => Node::tmp_load()?,
            false => Node::new(),
        }
    ));

    Ok(())
}

pub async fn full_node_main(load: bool) -> Result<()>{
    info!("Starting full Node ...");
    Ok(())
}

struct Config{
    version: usize,
    reward: usize,
    difficulty: usize,
}

impl Config{
    fn tmp_new() -> Self{
        Self { 
            version: 0, 
            reward: 10, 
            difficulty: 3 
        }
    }

    fn tmp_load() -> Self{
        Self::tmp_new()
    }
    
}