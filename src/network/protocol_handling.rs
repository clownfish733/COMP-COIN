use log::{info, warn};
use tokio::sync::{mpsc, RwLock};

use super::{
    connection::{ConnectionEvent, ConnectionType, ConnectionResponse},
    peers::PeerManager,
    protocol::NetMessage,
};

use crate::{node::{MineCommand, NetworkCommand, Node}};

use anyhow::Result;

use std::sync::Arc;




pub async fn protocal_handling(
    mut event_rx: mpsc::Receiver<ConnectionEvent>,
    peer_manager: Arc<RwLock<PeerManager>>,
    node: Arc<RwLock<Node>>,
    miner_tx: mpsc::Sender<MineCommand>,
    network_tx: mpsc::Sender<NetworkCommand>,
) -> Result<()>{

    while let Some(event) = event_rx.recv().await{
        let peer = event.network_address;

        match event.event{
            ConnectionType::Close => {
                info!("Closed: {}", &peer);
                let mut peer_manager_write = peer_manager.write().await;
                peer_manager_write.remove(&peer);
            }
            ConnectionType::Message(msg) => {
                info!("Received: {:?}", NetMessage::from_bytes(msg.clone()));
                match NetMessage::from_bytes(msg){
                    
                    Ok(NetMessage::GetBlock(index)) => {
                        let block_chain = node.read().await.block_chain.clone();
                        
                        if let Some(block) = block_chain.get(index){
                            let response = ConnectionResponse::message(
                                NetMessage::NewBlock(
                                    block.clone()
                                ).to_bytes()
                            );

                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e)
                            } 
                        }
                    }

                    Ok(NetMessage::NewBlock(block)) => {
                        let is_new = {
                            let node_read = node.read().await;
                            node_read.is_new_block(&block).await

                        };

                        if is_new{
                            {
                                let mut node_write = node.write().await;
                                node_write.add_block(&block).await;
                            }

                            if let Err(e) = miner_tx.send(MineCommand::UpdateBlock).await{
                                warn!("Error sending message to: {}: {}", &peer, e);
                            }

                            {   
                                info!("Requesting next block");
                                let response = ConnectionResponse::message(
                                    NetMessage::NewBlock(
                                        block.clone()
                                    ).to_bytes()
                                );
                                let peer_manager_read = peer_manager.read().await;
                                peer_manager_read.broadcast(response).await;

                                let response = ConnectionResponse::message(
                                    NetMessage::GetBlock(
                                        node.read().await.get_next_height()
                                    ).to_bytes()
                                );
                                info!("Requesting new block");
                                if let Err(e) = peer_manager_read.send(&peer, response).await{
                                    warn!("Error sending message to: {}: {}", &peer, e);
                                }
                            }
                        }else{
                            warn!("Old block received");
                        }


                    }
                    Ok(NetMessage::Verack(verack)) => {
                        if verack.index == 0{
                            let response = ConnectionResponse::message(
                                NetMessage::verack(
                                    1, 
                                    node.read().await.height
                                ).to_bytes()
                            );
                            {
                                let peer_manager_read = peer_manager.read().await;
                                if let Err(e) = peer_manager_read.send(&peer, response).await{
                                    warn!("Error sending message to: {}: {}", &peer, e);
                                }
                            }
                        }

                        let response = ConnectionResponse::message(
                            NetMessage::GetBlock(
                                match (verack.height, node.read().await.height){
                                    (Some(_h), None) => 0,
                                    (None, _) => continue,
                                    (Some(height), Some(node_height)) =>{
                                        if height > node_height{
                                            height + 1
                                        }else{
                                            continue;
                                        }
                                    }
                                }
                            ).to_bytes()
                        );

                        {
                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e);
                            }
                        }
                    }

                    Ok(NetMessage::Transaction(transaction)) => {
                        if !node.read().await.is_new_transaction(&transaction).await{continue;}
                        {
                            let mut node_write = node.write().await;
                            node_write.add_transaction(transaction.clone()).await;
                        }
                        
                        {
                            let response = ConnectionResponse::message(
                                NetMessage::Transaction(
                                    transaction.clone()
                                ).to_bytes()
                            );
                            let peer_manager_read = peer_manager.read().await;
                            peer_manager_read.broadcast(response).await;
                        }


                    }
                    Ok(NetMessage::GetInv) => {
                        let response = ConnectionResponse::message(
                            NetMessage::Inv(
                                node.read().await.mempool.clone()
                            ).to_bytes()
                        );

                        {
                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e);
                            }
                        }

                    }

                    Ok(NetMessage::Inv(mempool)) => {
                        node.write().await.update_mempool(mempool).await;
                    }

                    Ok(NetMessage::GetPeers) => {
                        let response = ConnectionResponse::message(
                            NetMessage::Peers(
                                peer_manager.read().await.get_peers()
                            ).to_bytes()
                        );
                        {
                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e);
                            }
                        }
                        {
                            let mut peer_manager_write = peer_manager.write().await;
                            peer_manager_write.reset_tick(peer);
                        }
                    }

                    Ok(NetMessage::Peers(peers)) => {
                        for peer in peers{
                            if let Err(e) = network_tx.send(NetworkCommand::Connect(peer)).await{
                                warn!("Error sending network command: {}", e);
                            }
                        }
                    }

                    Ok(NetMessage::Ping) => {
                        let response = ConnectionResponse::message(
                            NetMessage::Pong.to_bytes()
                        );
                        {
                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e);
                            }
                        }
                    }
                    Ok(NetMessage::Pong) => {},

                    Err(e) => {
                        warn!("Unable to deserialize message from: {} : {}", &peer, e);
                    }
                }
            }
        }
    }
    Ok(())
}