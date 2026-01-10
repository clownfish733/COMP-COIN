use std::net::SocketAddr;

use anyhow::Result;

use serde::{Serialize, Deserialize};

use crate::{
    block::{
        Block, Transaction, Mempool
    }
};

#[derive(Serialize, Deserialize)]
pub enum NetMessage{
    GetBlock(usize),
    NewBlock(Block),
    Verack(Verack),
    Transaction(Transaction),
    GetInv,
    Inv(Mempool),
    GetPeers,
    Peers(Vec<SocketAddr>),
    Ping,
    Pong,
}

impl NetMessage{
    pub fn verack(index: usize, height: Option<usize>) -> Self{
        Self::Verack(Verack::new(index, height))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Verack{
    pub index: usize,
    pub height: Option<usize>,
}

impl Verack{
    fn new(index: usize, height: Option<usize>) -> Self{
        Self { 
            index, 
            height
        }
    }
}


impl NetMessage{
    pub fn to_bytes(&self) -> Vec<u8>{
        postcard::to_allocvec(self).expect("Failed to serialize")
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self>{
        Ok(postcard::from_bytes(&bytes)?)
    }
}