use std::sync::{Arc, atomic::{self, AtomicBool}};

use log::{info, warn};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc; 

use crate::{node::NetworkCommand, utils::{generate_nonce, get_timestamp, sha256}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block{
    height: usize,
    nonce: Vec<u8>,
    time_stamp: usize,
    difficulty: usize,
}

impl Block{

    pub fn new(
        height: usize, 
        difficulty: usize
    ) -> Self{
        Self{
            height,
            nonce: generate_nonce(),
            time_stamp: get_timestamp(),
            difficulty
        }
    }

    pub fn is_valid(&self) -> bool{
        return true
    }

    pub fn get_height(&self) -> usize{
        return self.height
    }

    fn get_difficulty(&self) -> usize{
        self.difficulty
    }

    fn meets_diffculty(hash: Vec<u8>, difficulty: usize) -> bool{
        hash.iter().take(difficulty).all(|&x| x == 0 )
    }

    fn to_bytes(&self) -> Vec<u8>{
        postcard::to_allocvec(self).expect("Failed to serialize block")
    }

    fn calculate_hash(&self) -> Vec<u8>{
        sha256(self.to_bytes())
    }

    fn update_nonce(&mut self){
        self.nonce = generate_nonce();
    }

    pub fn mine(
        &mut self, 
        stop: Arc<AtomicBool>, 
        id: usize,
        network_tx: mpsc::Sender<NetworkCommand>
    ){
        while !stop.load(atomic::Ordering::Relaxed){

            self.update_nonce();
            if Block::meets_diffculty(self.calculate_hash(), self.get_difficulty()){
                info!("Mined block: {}", self.get_height());
                if let Err(e) = network_tx.try_send(NetworkCommand::Block(self.clone())){
                    warn!("Unable to communicate on network channel: {}", e);
                }
            }
        }

    }



}