const DIFFICULTY: usize = 3;

const FILE_PATH: &str = "configs/node.json";

use log::info;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use std::{
    net::{IpAddr}, 
    sync::Arc,
    fs::File,
};

use tokio::sync::RwLock;

use crate::{
    block::{Block, Mempool, Transaction, UTXOS, Wallet}, 
    ui::{NodeStatus, UserStatus}, utils::{get_global_ip, get_local_ip}
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

    pub async fn load() -> Result<Self>{
        let file = File::open(FILE_PATH)?;
        let node_data: NodeSerde = serde_json::from_reader(file)?;
        Ok(Self { 
            height: node_data.height, 
            mempool: node_data.mempool, 
            block_chain: node_data.block_chain, 
            config: node_data.config, 
            utxos: Arc::new(RwLock::new(node_data.utxos)), 
            wallet: node_data.wallet 
        })
    }

    pub async fn save(&self) -> Result<()>{
        let utxos = self.utxos.read().await.clone();

        let node_data = NodeSerde{
            height: self.height,
            mempool: self.mempool.clone(),
            block_chain: self.block_chain.clone(),
            config: self.config.clone(),
            utxos,
            wallet: self.wallet.clone()
        };
        let file = File::create(FILE_PATH)?;
        serde_json::to_writer(file, &node_data)?;
        Ok(())
    }

    pub async fn initialise(port: usize) -> Result<Self>{
        match env::args().nth(2).as_deref(){
            Some("load") => Ok(Self::load().await.expect("Unable to load node")), 
            Some("new") => Ok(Self::new(port).await),
            Some(arg) => return Err(anyhow!("invalid arguement: '{}' expected 'new' or 'load'", arg)),
            None => return Err(anyhow!("Missing arguement: expected: 'load' or 'new"))
        }
    }

    pub fn get_height(&self) -> Option<usize>{
        self.height
    }

    fn get_reward(&self) -> usize{
        self.config.reward
    }

    pub fn get_version(&self) -> usize{
        self.config.version
    }
    
    fn get_difficulty(&self) -> usize{
        self.config.difficulty
    }

    fn incr_height(&mut self){
        if let Some(height) = self.height{
            self.height = Some(height + 1);
        }else{
            self.height = Some(0);
        }
    }

    pub async fn is_new_block(&self, block: &Block) -> bool{

        self.utxos.read().await.validate_block(&block, self.get_reward())
        && block.get_height() == self.get_next_height()

    }
    
    fn get_prev_hash(&self) -> Vec<u8>{
        if let Some(block) = self.block_chain.last(){
            block.calculate_hash()
        } else{
            b"Hello World".to_vec()
        }
    }

    pub async fn add_block(&mut self, block: &Block){
        info!("Adding new block");
        self.block_chain.push(block.clone());
        self.mempool.add_block(block);
        self.wallet.add_block(block);
        self.utxos.write().await.add_block(block);
        self.incr_height();
    }

    pub async fn is_new_transaction(&self, transaction: &Transaction) -> bool{
        if !self.utxos.read().await.validate_pending_transaction(transaction){return false}

        !self.mempool.contains(transaction)
    }

    pub async fn add_transaction(&mut self, transaction: Transaction){
        let fee = self.utxos.read().await.calculate_fee(&transaction);
        self.mempool.add_transaction(transaction, fee);
    }

    pub async fn update_mempool(&mut self, mempool: Mempool){
        if self.utxos.read().await.validate_mempool(&mempool){
            self.mempool.update(mempool);
        }
    }

    pub async fn get_next_block(&mut self) -> Block{
        let transactions = self.mempool.get_next_transactions(
            Arc::clone(&self.utxos), 
            self.wallet.get_public_key(), 
            self.get_reward(), 
            self.get_version()
        ).await;
        
        Block::new(
            self.get_next_height(), 
            self.get_difficulty(), 
            self.get_version(), 
            transactions, 
            self.get_prev_hash()
        )
    }

    pub fn get_next_height(&self) -> usize{
        if let Some(height) = self.get_height(){
            height + 1
        }else{
            0
        }
    }

    pub fn get_user_status(&self) -> UserStatus{
        UserStatus::new(
            self.wallet.get_funds(),
            self.wallet.get_public_key().to_hex()
        )
    }

    pub fn get_node_status(&self) -> NodeStatus{
        NodeStatus::new(
            match self.get_height() {
                Some(height) => height,
                None => 888888
             }, 
             self.mempool.size(), 
             self.get_difficulty())
    }
}


#[derive(Clone, Deserialize, Serialize, Debug)]
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


#[derive(Serialize, Deserialize, Debug)]
struct NodeSerde {
    height: Option<usize>,
    mempool: Mempool,
    block_chain: Vec<Block>,
    config: Config,
    utxos: UTXOS,
    wallet: Wallet,
}