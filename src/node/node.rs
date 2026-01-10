const DIFFICULTY: usize = 3;

use anyhow::{Result, anyhow};

use std::{net::{IpAddr, SocketAddr}, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    block::{
        Block, Mempool, Transaction, UTXOS, Wallet
    },
    utils::{get_global_ip, get_local_ip}
};

use std::env;

pub struct Node{
    pub height: Option<usize>,
    pub mempool: Mempool,
    pub block_chain: Vec<Block>,
    pub config: Config,
    pub utxos: Arc<RwLock<UTXOS>>,
    pub wallet: Wallet,
}

impl Node{
    async fn new(port: usize) -> Self{
        Self {
            height: None, 
            mempool: Mempool::new(), 
            block_chain: Vec::new(), 
            config:Config::tmp_load(port).await, 
            utxos: Arc::new(RwLock::new(UTXOS::new())), 
            wallet: Wallet::new() 
        }
    }

    async fn tmp_load(port: usize) -> Result<Self>{
        Ok(Self::new(port).await)
    }

    pub async fn initialise(port: usize) -> Result<Self>{
        match env::args().nth(2).as_deref(){
            Some("load") => Self::tmp_load(port).await, 
            Some("new") => Ok(Self::new(port).await),
            Some(arg) => return Err(anyhow!("invalid arguement: '{}' expected 'new' or 'load'", arg)),
            None => return Err(anyhow!("Missing arguement: expected: 'load' or 'new"))
        }
    }

    pub fn is_new_block(&self, block: &Block) -> bool{
        match block.get_height(){
            0 => {
                if self.height != None {
                    return false
                };
            }

            height => {
                match self.height{
                    None => {
                        return false
                    }
                    Some(node_height) => {
                        if height != node_height + 1{
                            return false
                        }
                    }
                }
            }
        } 

        block.is_valid()

    }

    pub fn add_block(&mut self, block: &Block){
        self.block_chain.push(block.clone())
    }

    pub fn is_new_transaction(&self, transaction: &Transaction) -> bool{
        transaction.is_valid()
    }

    pub fn add_transaction(&mut self, transaction: Transaction){
        todo!("Implement adding transaction to node")
    }

    pub fn update_mempool(&mut self, mempool: Mempool){
        todo!("Implement updating mempool")
    }

    pub fn get_next_block(&self) -> Block{
        match self.height{
            Some(height) => Block::new(height + 1, self.config.difficulty),
            None => Block::new(0, self.config.difficulty)
        }
    }
}


#[derive(Clone)]
pub struct Config{
    version: usize,
    reward: usize,
    difficulty: usize,
    port: usize,
    local_ip: IpAddr,
    global_ip: IpAddr
}

impl Config{
    async fn tmp_new() -> Self{
        Self { 
            version: 0, 
            reward: 10, 
            difficulty: DIFFICULTY, 
            port: 8080,
            local_ip: get_local_ip().unwrap(),
            global_ip: get_global_ip().await.unwrap()
        }
    }

    async fn tmp_load(port: usize) -> Self{
        let mut config = Self::tmp_new().await;
        config.port = port;
        config
    }

    pub fn get_port(&self) -> usize{
        self.port
    }

    pub fn get_local_ip(&self) -> IpAddr{
        self.local_ip
    }

    pub fn get_global_ip(&self) -> IpAddr{
        self.global_ip
    }
    
}