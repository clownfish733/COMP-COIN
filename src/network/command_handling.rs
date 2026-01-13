use std::sync::{Arc};

use log::{info, warn, error};

use tokio::{net::TcpStream, sync::{RwLock, mpsc}};

use anyhow::Result;

use super::{
    connection::{ConnectionEvent, ConnectionResponse, connection_receiver, connection_sender},
    peers::PeerManager,
    protocol::NetMessage,
    server::CHANNEL_SIZE,
};

use crate::{
    node::{MineCommand, NetworkCommand, Node}
};

pub async fn command_handling(
    mut network_rx: mpsc::Receiver<NetworkCommand>,
    peer_manager: Arc<RwLock<PeerManager>>,
    node: Arc<RwLock<Node>>,
    miner_tx: mpsc::Sender<MineCommand>,
    event_tx: mpsc::Sender<ConnectionEvent>
) -> Result<()>{
    while let Some(command) = network_rx.recv().await{
        match command{
            NetworkCommand::Block(block) => {
                if node.read().await.is_new_block(&block).await{
                    info!("Adding new block");
                    {
                        let mut node_writer = node.write().await;
                        node_writer.add_block(&block).await;
                    }
                    {
                        let response = ConnectionResponse::message(
                            NetMessage::NewBlock(block).to_bytes()
                        );

                        let peer_manager_read = peer_manager.read().await;
                        peer_manager_read.broadcast(response).await;
                    }
        
                }else{
                    warn!("Is Old Block");
                }

                if let Err(e) = miner_tx.send(MineCommand::UpdateBlock).await{
                    error!("Error sending mining command: {}", e);
                }

            }

            NetworkCommand::Connect(peer) => {
                let should_connect = {
                    let peer_manager_read = peer_manager.read().await;
                    let config = node.read().await.config.clone();

                    !peer_manager_read.contains(&peer) 
                    && config.get_local_ip() != peer.ip() 
                    && config.get_global_ip() != peer.ip()
                };

                if should_connect{
                    if let Ok(stream) = TcpStream::connect(&peer).await{
                        info!("Connected to: {}", &peer);

                        let (response_tx, response_rx) = mpsc::channel::<ConnectionResponse>(CHANNEL_SIZE);

                        {
                            let mut peer_manager_write = peer_manager.write().await;
                            peer_manager_write.insert(peer.clone(), response_tx);
                        }

                        let (reader, writer) = stream.into_split();

                        tokio::spawn({
                            let event_tx = event_tx.clone();
                            let peer = peer.clone();
                            async move {
                                connection_receiver(reader, peer, event_tx)
                                .await
                                .expect("Error connection sender")
                            }}
                        );

                        tokio::spawn({
                            let peer = peer.clone();
                            async move {
                                connection_sender(writer, response_rx, peer)
                                .await
                                .expect("Error connection sender")
                            }
                        });

                        {
                            let height = node.read().await.height;
                            let response = ConnectionResponse::message(
                                NetMessage::verack(0, height).to_bytes()
                            );
                            let peer_manager_read = peer_manager.read().await;
                            if let Err(e) = peer_manager_read.send(&peer, response).await{
                                warn!("Error sending message to: {}: {}", &peer, e)
                            }
                        }
                    }else{
                        warn!("Failed to connect to: {}", &peer);
                    }
                }
            }

            NetworkCommand::Transaction(transaction) => {
                let is_new = {
                    node.read().await.is_new_transaction(&transaction).await
                };
                if is_new{
                    {
                    let mut node_write =  node.write().await;
                    node_write.add_transaction(transaction.clone()).await;
                    }

                    {
                        let response = ConnectionResponse::message(
                            NetMessage::Transaction(transaction).to_bytes()
                        );
                        let peer_manager_read = peer_manager.read().await;
                        peer_manager_read.broadcast(response).await;
                    }
                } else{
                    warn!("Is old Transaction");
                }

            }
        }
    }

    Ok(())
}