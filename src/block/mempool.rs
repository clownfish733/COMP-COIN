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
}


#[derive(Clone, Debug)]
pub struct Mempool(HeapSet<TransactionWithFee>);

impl Mempool {
    pub fn new() -> Self{
        Self(HeapSet::new())
    }

    fn push(&mut self, tx: Transaction, fee: usize){
        self.0.push(TransactionWithFee { transaction: tx, fee });
    }

    fn from_vec(v: Vec<TransactionWithFee>) -> Result<Self>{
        Ok(Self(HeapSet::from_vec(v)))
    }

    fn to_vec(&self) -> Vec<TransactionWithFee>{
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

    fn add_block(&mut self, block: Block){
        self.remove(block.get_transactions());
    }

    async fn add_transaction(
        &mut self, 
        transaction: Transaction, 
        utxos: Arc<RwLock<UTXOS>>
    ){
        let fee = {
            let utxos_read = utxos.read().await;
            utxos_read.calculate_fee(&transaction)
        };
        self.push(transaction, fee);
    }

    async fn get_next_transactions(&mut self, utxos: Arc<RwLock<UTXOS>>, public_key: PublicKey) -> Vec<Transaction>{
        let mut txs = Vec::new();
        let mut invalid_txs = Vec::new();

        let mut temp_mempool = self.0.clone();

        while txs.len() < TRANSACTIONS_PER_BLOCK{
            let Some(TransactionWithFee{transaction: tx, fee: _fee}) = temp_mempool.pop() else{
                self.remove(invalid_txs);
                return txs
            };

            let is_valid = {
                let utxos_read = utxos.read().await;
                utxos_read.validate_pending_transaction(tx.clone())
            };

            if is_valid{
                txs.push(tx);
            }else{
                invalid_txs.push(tx);
            }
        }
        self.remove(invalid_txs);
        txs
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
struct TransactionWithFee{
    transaction: Transaction,
    fee: usize,
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