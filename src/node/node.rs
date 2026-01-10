use anyhow::{Result, anyhow};
use crate::{
    block::{
        Mempool,
        Block,
        UTXOS,
        Wallet,
    }
};

use std::env;

pub struct Node{
    height: usize,
    pub mempool: Mempool,
    pub block_chain: Vec<Block>,
    pub config: Config,
    pub utxos: UTXOS,
    pub wallet: Wallet,
}

impl Node{
    fn new() -> Self{
        Self {
            height: 0, 
            mempool: Mempool::new(), 
            block_chain: Vec::new(), 
            config:Config::tmp_load(), 
            utxos: UTXOS::new(), 
            wallet: Wallet::new() 
        }
    }

    fn tmp_load() -> Result<Self>{
        Ok(Self::new())
    }

    pub fn initialise() -> Result<Self>{
        match env::args().nth(2).as_deref(){
            Some("load") => Self::tmp_load(), 
            Some("new") => Ok(Self::new()),
            Some(arg) => return Err(anyhow!("invalid arguement: '{}' expected 'new' or 'load'", arg)),
            None => return Err(anyhow!("Missing arguement: expected: 'load' or 'new"))
        }
    }
}


pub struct Config{
    version: usize,
    reward: usize,
    difficulty: usize,
    port: usize,
}

impl Config{
    fn tmp_new() -> Self{
        Self { 
            version: 0, 
            reward: 10, 
            difficulty: 3, 
            port: 8080
        }
    }

    fn tmp_load() -> Self{
        Self::tmp_new()
    }

    pub fn get_port(&self) -> usize{
        self.port
    }
    
}