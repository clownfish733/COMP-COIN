use std::net::SocketAddr;

use crate::block::{Block, Transaction};

pub enum NetworkCommand{
    Block(Block),
    Transaction(Transaction),
    Connect(SocketAddr)

}

pub enum MineCommand{
    Stop,
    UpdateBlock,
}