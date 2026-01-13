use std::sync::{Arc, atomic::{self, AtomicBool}};

use log::{info, warn};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc; 

use super::transaction::Transaction;

//delete private key import later
use crate::{
    block::keys::{PrivateKey}, node::NetworkCommand, utils::{format_number, generate_nonce, get_timestamp, sha256}
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block{
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct BlockHeader{
    prev_hash: Vec<u8>,
    merkle_root: Vec<u8>,
    timestamp: usize,
    difficulty: usize,
    nonce: Vec<u8>,
    version: usize,
    height: usize,
}

impl BlockHeader{
    fn new(
        prev_hash: Vec<u8>,
        merkle_root: Vec<u8>,
        difficulty: usize,
        version: usize,
        height: usize,
    ) -> Self{
        Self { 
            prev_hash, 
            merkle_root, 
            timestamp: get_timestamp(), 
            difficulty, 
            nonce: generate_nonce(),
            version, 
            height 
        }
    }   

    fn get_height(&self) -> usize{
        self.height
    }

    fn get_difficulty(&self) -> usize{
        self.difficulty
    }

    fn update_nonce(&mut self){
        self.nonce = generate_nonce();
    }
}

impl Block{

    pub fn new(
        height: usize, 
        difficulty: usize,
        version: usize,
        transactions: Vec<Transaction>,
        prev_hash: Vec<u8>
    ) -> Self{
        Self { 
            header: BlockHeader::new(
                prev_hash, 
                Self::get_merkle_root(&transactions), 
                difficulty, 
                version, 
                height), 
            transactions, 
        }
    }

    pub fn get_height(&self) -> usize{
        return self.header.get_height()
    }

    fn get_difficulty(&self) -> usize{
        self.header.get_difficulty()
    }

    pub fn get_transactions(&self) -> Vec<Transaction>{
        self.transactions.clone()
    }

    fn meets_diffculty(hash: Vec<u8>, difficulty: usize) -> bool{
        hash.iter().take(difficulty).all(|&x| x == 0 )
    }

    fn to_bytes(&self) -> Vec<u8>{
        postcard::to_allocvec(self).expect("Failed to serialize block")
    }

    pub fn calculate_hash(&self) -> Vec<u8>{
        sha256(self.to_bytes())
    }

    fn update_nonce(&mut self){
        self.header.update_nonce();
    }

    pub fn mine(
        &mut self, 
        stop: Arc<AtomicBool>, 
        id: usize,
        network_tx: mpsc::Sender<NetworkCommand>
    ){
        info!("Thread: {} started mining", &id);

        let mut count: usize = 1;

        while !stop.load(atomic::Ordering::Relaxed){

            self.update_nonce();
            if Block::meets_diffculty(self.calculate_hash(), self.get_difficulty()){
                info!("Mined block: {}", self.get_height());
                if let Err(e) = network_tx.try_send(NetworkCommand::Block(self.clone())){
                    warn!("Unable to communicate on network channel: {}", e);
                }
            }
            
            if count%250_000 == 0 && id == 0{
                info!("Each thread tried: {} blocks", format_number(count));
            }
            count += 1;
        }

    }

    fn get_merkle_root(transactions: &Vec<Transaction>) -> Vec<u8>{
        Self::rec_merkle_root(transactions.iter().map(|tx| tx.to_bytes()).collect())
    }

    fn rec_merkle_root(transactions: Vec<Vec<u8>>) -> Vec<u8>{
        match transactions.len(){
            0 => sha256(b"Hello World".to_vec()),

            1 => {
                let message = transactions[0].repeat(2);

                sha256(message)
            }

            2 => {
                let message = transactions.iter()
                    .take(2)
                    .flatten()
                    .copied()
                    .collect();

                sha256(message)
            }

            _ => {
                let mut stack = Vec::new();
                let mut message: Vec<u8>;
                for pair in transactions.chunks(2) {
                    if pair.len() == 1{
                        message = pair[0].repeat(2);
                    }
                    else{
                        message = pair.concat();
                    }
                    stack.push(sha256(message.clone()));
                    message.clear();
                }

                Self::rec_merkle_root(stack)
            }
        }
    }


    //delete later

    pub fn temp_block() -> Self{
        Self { header: BlockHeader { 
            prev_hash: b"hello world".to_vec(), 
            merkle_root: b"Merkle_root".to_vec(), 
            timestamp: 100, 
            difficulty: 3, 
            nonce: b"nonce".to_vec(), 
            version: 0, 
            height: 0 
        }, 
        transactions: vec![Transaction::reward(100, PrivateKey::new().get_public_key(), 0)]
    }
    }
}