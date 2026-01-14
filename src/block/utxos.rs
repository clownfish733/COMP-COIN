use std::{collections::HashMap};

use log::{info, warn};
use serde::{Deserialize, Serialize};

use super::{
    transaction::{TxOutput, Transaction, TxInput},
    block::Block,
    script::Script,
    mempool::{Mempool,TransactionWithFee}
};

#[derive(Clone, Debug)]
pub struct UTXOS(HashMap<(Vec<u8>, usize), TxOutput>);

impl UTXOS{
    pub fn new() -> Self{
        Self(HashMap::new())
    }

    fn insert(&mut self, hash: Vec<u8>, index: usize, utxo: TxOutput){
        self.0.insert((hash, index), utxo);
    }

    fn remove(&mut self, hash: Vec<u8>, index: usize){
        self.0.remove(&(hash, index));
    }

    fn get(&self, hash: &Vec<u8>, index: usize) -> Option<TxOutput>{
        self.0.get(&(hash.clone(), index)).cloned()
    }

    fn add_transaction(&mut self, tx: Transaction){
        let hash = tx.get_hash();

        for input in tx.inputs{
            self.remove(input.prev, input.output_index);
        }
        
        for (index, output) in tx.outputs.iter().enumerate(){
            self.insert(hash.clone(), index, output.clone());        
        }
    }

    pub fn add_block(&mut self, block: &Block){
        for tx in block.get_transactions(){
            self.add_transaction(tx);
        }
    }
    fn validate_scripts(&self, tx: &Transaction) -> bool{
        for (index, input) in tx.inputs.iter().enumerate(){
            let Some(utxo) = self.get(&input.prev, input.output_index) else{
                warn!("Could not get utxo: {}, {}", hex::encode(&input.prev), input.output_index);
                return false
            };
            
            if !Script::concat(&input.unlocking_script, &utxo.locking_script).validate(&tx, index, &utxo){
                warn!("script validation failed for script: {:?}", Script::concat(&input.unlocking_script, &utxo.locking_script));
                return false 
            }
        }

        true
    }

    fn validate_confirmed_transaction(&self, tx: &Transaction) -> bool{
        if !self.validate_scripts(&tx){
            warn!("1"); return false}

        self.get_input_value(tx.inputs.clone()) == UTXOS::get_output_value(tx.outputs.clone())
    }

    pub fn validate_pending_transaction(&self, tx: &Transaction) -> bool{
        if !self.validate_scripts(&tx){
            tx.debug();
            warn!("Invalid script");
            return false
        }
        
        self.get_input_value(tx.inputs.clone()) >= UTXOS::get_output_value(tx.outputs.clone())
    }

    fn get_input_value(&self, inputs: Vec<TxInput>) -> usize{
        let mut input_amount = 0;

        for input in inputs{
            input_amount += self.get(&input.prev, input.output_index).unwrap().value
        }

        input_amount
    }

    fn get_output_value(outputs: Vec<TxOutput>) -> usize{
        outputs.iter().map(|utxo| utxo.value).sum()
    }

    pub fn is_coinbase(transaction: &Transaction, reward: usize) -> bool{
        transaction.inputs.len() == 0 
        && transaction.outputs.len() == 1
        && transaction.outputs[0].value == reward
    }

    pub fn validate_block(&self, block: &Block, reward: usize) -> bool{
        let txs = block.get_transactions();
        let Some(coinbase) = txs.get(0)else{
            warn!("Missing coinbase");
            return false
        };
        if !Self::is_coinbase(coinbase, reward){
            warn!("Invalid coinbase");
            return false
        }

        for tx in &block.get_transactions()[1..]{
            if !self.validate_confirmed_transaction(tx){
                warn!("Invalid transction");
                return false
            }
        }

        true
    }

    pub fn calculate_fee(&self, transaction: &Transaction) -> usize{
        self.get_input_value(transaction.inputs.clone()) - Self::get_output_value(transaction.outputs.clone())
    }

    pub fn validate_mempool(&self, mempool: &Mempool) -> bool{
        for TransactionWithFee{transaction, fee} in mempool.to_vec(){
            if !self.validate_pending_transaction(&transaction){return false}
            if self.calculate_fee(&transaction) != fee {return false}
        }
        return true
    }
}

impl Serialize for UTXOS{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let serde_utxos: HashMap<String, TxOutput> = self.0.iter().map(
            |((hash, index), output)|
            (format!("{}:{}", hex::encode(hash.clone()), index.clone()), output.clone())
        ).collect();

        serde_utxos.serialize(serializer)
    }  
}

impl <'de>Deserialize<'de> for UTXOS{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let serde_utxos = HashMap::<String, TxOutput>::deserialize(deserializer)?;
        let utxos = serde_utxos.iter().map(
            |(key, output)|{

            let (hash, index) = key.split_once(":").expect("No colon found for utxos key"); 
            let hash = hex::decode(&hash).map_err(serde::de::Error::custom)?;
            let index: usize = index.parse().expect("index not a number");
            Ok(((hash, index.clone()), output.clone()))
            }
            
        ).collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Self(utxos))
    }
}