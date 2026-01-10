use tokio::{
    sync::{mpsc}
};

use anyhow::{Result, anyhow};

use log::{warn};

use std::{
    net::SocketAddr,
    collections::{
        HashMap,
        hash_map::Iter,
    },
};

use super::connection::{ 
    ConnectionResponse
};

#[derive(Clone)]
struct PeerInfo{
    tx: mpsc::Sender<ConnectionResponse>,
}

impl PeerInfo{
    fn new(tx: mpsc::Sender<ConnectionResponse>) -> Self{
        Self { 
            tx 
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

    pub fn get(&self, network_address: &SocketAddr) -> Option<PeerInfo>{
        self.0.get(network_address).cloned()
    }

    pub fn contains(&self, network_address: &SocketAddr) -> bool{
        self.0.contains_key(network_address)
    }

    pub fn iter(&self) -> Iter<'_, SocketAddr, PeerInfo>{
        self.0.iter()
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
