use tokio::{
    sync::{mpsc, RwLock}
};

use anyhow::{Result, anyhow};

use log::{warn};

use std::{
    collections::{
        HashMap,
        hash_map::{IterMut, Iter}
    }, net::SocketAddr, sync::Arc,
};

use super::{
    connection::ConnectionResponse,
    protocol::NetMessage,
};

#[derive(Clone)]
struct PeerInfo{
    tx: mpsc::Sender<ConnectionResponse>,
    refresh_tick: usize,
}

impl PeerInfo{
    fn new(tx: mpsc::Sender<ConnectionResponse>) -> Self{
        Self { 
            tx,
            refresh_tick: 0,
        }
    }

    fn check_ticker(&mut self) -> bool{
        if self.refresh_tick == 3{
            self.refresh_tick = 0;
            return true
        }else{
            self.refresh_tick += 1;
            return false
        }
    }
}

#[derive(Clone)]
pub struct PeerManager(HashMap<SocketAddr, PeerInfo>);

impl PeerManager{
    pub fn new() -> Self{
        Self(
            HashMap::new()
        )
    }

    pub fn insert(&mut self, network_address: SocketAddr, tx: mpsc::Sender<ConnectionResponse>){
        self.0.insert(
            network_address,
            PeerInfo::new(tx) 
        );
    }

    pub fn remove(&mut self, network_address: &SocketAddr){
        self.0.remove(network_address);
    }

    fn get(&self, network_address: &SocketAddr) -> Option<PeerInfo>{
        self.0.get(network_address).cloned()
    }

    pub fn contains(&self, network_address: &SocketAddr) -> bool{
        self.0.contains_key(network_address)
    }

    fn iter_mut(&mut self) -> IterMut<'_, SocketAddr, PeerInfo>{
        self.0.iter_mut()
    }

    fn iter(&self) -> Iter<'_, SocketAddr, PeerInfo>{
        self.0.iter()
    }

    pub fn reset_tick(&mut self, peer: SocketAddr){
        if let Some(info) = self.0.get_mut(&peer){
            info.refresh_tick = 0;
        }else{
            warn!("Tried to refresh tick for peer not in peer manager");
        }
    }

    pub async fn send(
        &self, 
        network_address: &SocketAddr, 
        response: ConnectionResponse
    ) -> Result<()>{

        let tx = match self.get(network_address){
            Some(peer_info) => peer_info.tx,
            None => return Err(anyhow!("network address: {} not in peer manager", network_address))
        };

        tx.send(response).await?;
        Ok(())
    }

    //returns option of list of failed network addressess
    pub async fn broadcast(
        &self,
        response: ConnectionResponse
    ) -> Option<Vec<SocketAddr>>{
        let mut failed_network_address = Vec::new();


        for (network_address, peer_info) in self.iter(){
            if let Err(e) =  peer_info.tx.send(response.clone()).await{
                warn!("Unable to send to: {} : {}", network_address, e);
                failed_network_address.push(*network_address)
            }
        }

        if failed_network_address.is_empty(){
            return None
        }else{
            Some(failed_network_address)
        }
    }

    pub fn get_peers(&self) -> Vec<SocketAddr>{
        self.0.keys().cloned().collect()
    }

}

#[allow(unused)]
pub async fn update_peers(peer_manager: Arc<RwLock<PeerManager>>){
    let response = ConnectionResponse::message(
        NetMessage::GetPeers.to_bytes()
    );
    loop {
        // Collect peers that need updates
        let peers_to_update: Vec<SocketAddr> = {
            let mut pm = peer_manager.write().await;
            pm.iter_mut()
                .filter_map(|(peer, info)| {
                    if info.check_ticker() {
                        Some(*peer)
                    } else {
                        None
                    }
                })
                .collect()
        }; 
        
        // Send messages without holding any lock
        for peer in peers_to_update {
            let pm = peer_manager.read().await;
            if let Err(e) = pm.send(&peer, response.clone()).await {
                warn!("Error sending to: {}", peer);
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}