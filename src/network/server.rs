pub const CHANNEL_SIZE: usize = 100;

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

use super::{
    connection::{ConnectionEvent,ConnectionResponse, connection_receiver, connection_sender},
    protocol_handling::protocal_handling,
    command_handling::command_handling,
    peers::update_peers,
};

use super::peers::PeerManager;

pub async fn start_network_server(
    node: Arc<RwLock<Node>>,
    miner_tx: mpsc::Sender<MineCommand>,
    network_rx: mpsc::Receiver<NetworkCommand>,
    network_tx: mpsc::Sender<NetworkCommand>
) -> Result<()>{

    info!("Started Network Server");

    //initiating listener
    let port = node.read().await.config.get_port();
    let socket_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&socket_addr).await?;
    info!("Started listening on {}", socket_addr);


    let peer_manager = Arc::new(RwLock::new(PeerManager::new()));
    let (event_tx, event_rx) = mpsc::channel::<ConnectionEvent>(CHANNEL_SIZE);

    tokio::spawn({
        let peer_manager = Arc::clone(&peer_manager);
        let node = Arc::clone(&node);
        let miner_tx = miner_tx.clone();
        let network_tx = network_tx.clone();

        async {
            protocal_handling(
                event_rx, 
                peer_manager, 
                node, 
                miner_tx, 
                network_tx
            )
            .await
            .expect("Error protocol handling")
        }
    });

    tokio::spawn({
        let peer_manager = Arc::clone(&peer_manager);
        async{
            update_peers(peer_manager).await
        }
    });


    tokio::spawn({  
        let peer_manager = Arc::clone(&peer_manager);
        let node = Arc::clone(&node);
        let miner_tx = miner_tx.clone();
        let event_tx = event_tx.clone();
        async{
            command_handling(
                network_rx, 
                peer_manager, 
                node, 
                miner_tx, 
                event_tx
            )
            .await
            .expect("Error command handling")
        }
    });




    loop{
        let (stream, network_address) = listener.accept().await?;

        let (response_tx, response_rx) = mpsc::channel::<ConnectionResponse>(CHANNEL_SIZE);

        {
            let mut peer_manager_write = peer_manager.write().await;
            peer_manager_write.insert(network_address, response_tx);
        }

        let (reader, writer) = stream.into_split();

        tokio::spawn({
            let event_tx_clone = event_tx.clone();
            let peer = network_address.clone();
            async move {
                connection_receiver(reader, peer , event_tx_clone)
                .await
                .expect("Error connection receiver")
            }
        });

        tokio::spawn({
            let peer = network_address.clone();
            async move {
                connection_sender(writer, response_rx, peer)
                .await
                .expect("Error connection sender")
            }
        });

    }
}