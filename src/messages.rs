use std::net::SocketAddr;

use crate::{
    block::Block, 
    mempool::Mempool, 
    transaction::Transaction
};

pub enum Message{
    GetBlock,
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

pub struct Verack{
    index: usize,
    height: usize,
}