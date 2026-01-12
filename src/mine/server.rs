use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}}, 
    thread::{self, JoinHandle}
};

use anyhow::Result;

use log::{info};

use tokio::{
    sync::{RwLock, mpsc},
};

use crate::{
    node::{MineCommand, NetworkCommand, Node},
    block::Block,
};

fn spawn_threads(
    block: Block,
    stop: Arc<AtomicBool>,
    network_tx: mpsc::Sender<NetworkCommand>
) -> Vec<JoinHandle<()>>{
    let num_threads= thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    info!("Spawning: {} mining threads for block: {}", num_threads, block.get_height());

    let mut handles = Vec::new();

    for i in  0..num_threads{
        let mut block_clone = block.clone();
        let stop_clone = Arc::clone(&stop);
        let network_tx_clone = network_tx.clone();
        let handle = thread::spawn(move ||{ 
            block_clone.mine(stop_clone, i, network_tx_clone)
        });
        handles.push(handle);
    }   

    handles
}

pub async fn start_mining_server(
    node: Arc<RwLock<Node>>,
    mut miner_rx: mpsc::Receiver<MineCommand>,
    network_tx: mpsc::Sender<NetworkCommand>,
) -> Result<()>{
    info!("Started Mining Server");

    let mut stop = Arc::new(AtomicBool::new(false));

    let block = node.write().await.get_next_block().await;

    let mut handles = spawn_threads(
        block, 
        Arc::clone(&stop), 
        network_tx.clone()
    );

    
    while let Some(command) = miner_rx.recv().await{
        match command{
            MineCommand::Stop => {
                info!("Shutting down mining threads");
                stop.store(true, Ordering::Relaxed);
                break;
            }
            MineCommand::UpdateBlock => {
                stop.store(true, Ordering::Relaxed);
                
                for handle in handles{
                    handle.join().expect("Error joining handles");
                }

                stop = Arc::new(AtomicBool::new(false));
                let block = node.write().await.get_next_block().await;
                handles = spawn_threads(
                    block, 
                    Arc::clone(&stop), 
                    network_tx.clone()
                );
            }
        }
    };

    Ok(())
}