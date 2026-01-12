const TRANSACTIONS_PER_BLOCK: usize = 10;

use std::{
    collections::{BinaryHeap, HashSet},
    hash::{Hash},
    sync::Arc,
};

use serde::{Serialize, Deserialize}; 

use anyhow::Result;

use tokio::sync::RwLock;

use super::{
    transaction::Transaction,
    block::Block,
    utxos::UTXOS,
    keys::PublicKey,
};

#[derive(Clone, Debug)]
struct HeapSet<T>{
    heap: BinaryHeap<T>,
    elements: HashSet<T>,
}

impl<T: Ord + Clone + Hash> HeapSet<T>{
    fn new() -> Self{
        Self{
            heap: BinaryHeap::new(),
            elements: HashSet::new(),
        }
    }

    fn push(&mut self, item: T){
        if self.elements.insert(item.clone()) {
            self.heap.push(item);
        }
    }

    fn pop(&mut self) -> Option<T>{
        if let Some(item) = self.heap.pop(){
            self.elements.remove(&item);
            Some(item)
        } else {
            None
        }
    }

    fn contains(&self, item: T) -> bool{
        self.elements.contains(&item)
    }

    fn get_vec(&self) -> Vec<T>{
        self.heap.clone().into_vec()
    }

    fn from_vec(v: Vec<T>) -> Self{
        Self { 
            heap: BinaryHeap::from(v.clone()),
            elements: HashSet::from_iter(v)
        }
    }

    fn remove(&mut self, items: HashSet<T>){
        for item in items.iter(){
            self.elements.remove(item);
        }
        let vec: Vec<T> = self.heap.drain()
            .filter(|x| !items.contains(x))
            .collect();
        self.heap = BinaryHeap::from(vec)
    }

    fn update(&mut self, other: Self){
        for item in other.elements.iter(){
            self.push(item.clone())
        }
    }

    fn size(&self) -> usize{
        self.elements.len()
    }
}


#[derive(Clone, Debug)]
pub struct Mempool(HeapSet<TransactionWithFee>);

impl Mempool {
    pub fn new() -> Self{
        Self(HeapSet::new())
    }

    pub fn contains(&self, transaction: &Transaction) -> bool{
        self.0.contains(TransactionWithFee::new(transaction.clone(), 0))
    }

    fn push(&mut self, tx: Transaction, fee: usize){
        self.0.push(TransactionWithFee { transaction: tx, fee });
    }

    fn from_vec(v: Vec<TransactionWithFee>) -> Result<Self>{
        Ok(Self(HeapSet::from_vec(v)))
    }

    fn pop(&mut self) -> Option<TransactionWithFee>{
        self.0.pop()
    }

    pub fn to_vec(&self) -> Vec<TransactionWithFee>{
        self.0.get_vec()
    }

    fn remove(&mut self, transactions: Vec<Transaction>){
        let mut set = HashSet::new();
        for mut tx in transactions{
            tx.remove_fee();
            set.remove(&TransactionWithFee::new(tx, 0));
        }
        self.0.remove(set);
    }

    pub fn add_block(&mut self, block: &Block){
        self.remove(block.get_transactions());
    }

    pub fn add_transaction(
        &mut self, 
        transaction: Transaction, 
        fee: usize
    ){
        self.push(transaction, fee);
    }

    pub async fn get_next_transactions(
        &mut self, 
        utxos: Arc<RwLock<UTXOS>>, 
        public_key: PublicKey,
        reward: usize,
        version: usize,
    ) -> Vec<Transaction>{
        let mut txs = vec![Transaction::reward(reward, public_key.clone(), version)];
        let mut invalid_txs = Vec::new();

        let mut temp_mempool = self.clone();

        while txs.len() < TRANSACTIONS_PER_BLOCK{
            let Some(TransactionWithFee{transaction: mut tx, fee}) = temp_mempool.pop() else{
                temp_mempool.remove(invalid_txs);
                return txs
            };

            let is_valid = {
                let utxos_read = utxos.read().await;
                utxos_read.validate_pending_transaction(&tx)
            };

            if is_valid{
                tx.add_fee(public_key.clone(), fee);
                txs.push(tx);
            }else{
                invalid_txs.push(tx);
            }
        }
        self.remove(invalid_txs);
        txs
    }

    pub fn update(
        &mut self, 
        mempool: Mempool,
    ){
        self.0.update(mempool.0)
    }

    pub fn size(&self) -> usize{
        self.0.size()
    }
}

impl <'de>Deserialize<'de> for Mempool{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let txs = Vec::<TransactionWithFee>::deserialize(deserializer)?;
        use serde::de::Error;
        Mempool::from_vec(txs).map_err(D::Error::custom)
    }
}

impl Serialize for Mempool{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.to_vec().clone().serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionWithFee{
    pub transaction: Transaction,
    pub fee: usize,
}

impl TransactionWithFee{
    fn new(transaction: Transaction, fee: usize) -> Self{
        Self { 
            transaction, 
            fee 
        }
    }
}

impl PartialEq for TransactionWithFee{
    fn eq(&self, other: &Self) -> bool {
        self.transaction.get_hash() == other.transaction.get_hash()
    }
}

impl Eq for TransactionWithFee{}

impl Hash for TransactionWithFee{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.transaction.hash(state)
    }
}

impl Ord for TransactionWithFee{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fee.cmp(&other.fee)
    }
}

impl PartialOrd for TransactionWithFee{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}