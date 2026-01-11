const DIFFICULTY: usize = 3;

use anyhow::{Result, anyhow};

use std::{net::{IpAddr}, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    block::{
        Block, Mempool, Transaction, UTXOS, Wallet
    },
    utils::{get_global_ip, get_local_ip, sha256}
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

    pub fn get_height(&self) -> Option<usize>{
        self.height
    }

    fn incr_height(&mut self){
        if let Some(height) = self.height{
            self.height = Some(height + 1);
        }else{
            self.height = Some(0);
        }
    }

    pub fn is_new_block(&self, block: &Block) -> bool{

        block.is_valid() 
        && block.get_height() == self.get_next_height()

    }

    pub fn add_block(&mut self, block: &Block){
        self.block_chain.push(block.clone());
        self.incr_height();
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
        let prev_hash = match self.block_chain.last(){
            Some(last) => last.calculate_hash(),
            None => sha256(b"hello world".to_vec())
        };

        Block::new(
            self.get_next_height(),
            self.config.difficulty,
            self.config.version,
            vec![],
            prev_hash,
        )
    }

    pub fn get_next_height(&self) -> usize{
        if let Some(height) = self.height{
            height + 1
        }else{
            0
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