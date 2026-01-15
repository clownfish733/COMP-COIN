use std::collections::HashMap;

#[allow(unused_imports)]
use log::{info, warn};

use serde::{Deserialize, Serialize};

use super::{
    transaction::{TxOutput, Transaction},
    keys::{PublicKey,PrivateKey},
    transaction::{TransactionSpec,InputSpec, OutputSpec},
    block::Block,
};

#[derive(Debug, Clone)]
pub struct Wallet{
    utxos: HashMap<(Vec<u8>, usize), TxOutput>,
    funds: usize,
    public_key: PublicKey,
    private_key: PrivateKey
}

impl Wallet{
    pub fn new() -> Self{
        let private_key = PrivateKey::new();
        let public_key = private_key.get_public_key();

        Self{
            utxos: HashMap::new(),
            funds: 0,
            public_key,
            private_key,
        }
    }

    fn from_serde_wallet(serde_wallet: &SerdeWallet, utxos: HashMap<(Vec<u8>, usize), TxOutput>) -> Self{
        Self { 
            utxos, 
            funds: serde_wallet.funds, 
            public_key: serde_wallet.public_key.clone(), 
            private_key: serde_wallet.private_key.clone() 
        }
    }

    pub fn get_funds(&self) -> usize{
        self.funds
    }

    fn insert(&mut self, hash: Vec<u8>, index: usize, utxo: TxOutput){
        self.funds += utxo.value;
        self.utxos.insert((hash, index), utxo);
    }

    fn remove(&mut self, hash: Vec<u8>, index: usize){
        if let Some(output) = self.utxos.get(&(hash.clone(), index)){
            self.funds -= output.value;
            self.utxos.remove(&(hash, index));
        }
    }

    pub fn get_public_key(&self) -> PublicKey{
        self.public_key.clone()
    }

    fn add_transaction(&mut self, tx: Transaction){
        let hash = tx.get_hash();

        for input in tx.inputs{
            self.remove(input.prev, input.output_index);
        }

        for (index, output) in tx.outputs.iter().enumerate(){
            if output.locking_script.P2PKHLocking_get_public_key_hash().unwrap() 
                == self.public_key.to_hash(){
                self.insert(hash.clone(), index, output.clone());
            }
        }
    }

    pub fn add_block(&mut self, block: &Block){
        for transaction in block.get_transactions(){
            self.add_transaction(transaction);
        }
    }

    pub fn new_transaction(&self, version: usize, outputs: Vec<OutputSpec>, fee: usize) -> Transaction{
        let mut tx_spec = TransactionSpec::pre_inputs(
            version, 
            outputs, 
            self.public_key.clone(), 
            self.private_key.clone()
        );
        
        let spend: usize = tx_spec.outputs.iter().map(|utxo| utxo.value).sum();

        let mut utxos = self.utxos.clone();
        let mut spent: usize = 0;

        while spent < spend + fee{
            if let Some((hash, index)) = utxos.keys().next().cloned(){
                
                let utxo = utxos.remove(&(hash.clone(), index)).unwrap();
                spent += utxo.value;
                tx_spec.inputs.push(InputSpec::new(hash, index, utxo));
                
            }else{
                warn!("Insufficient funds");
                panic!();
            }
        }
        if spent > (spend + fee){
            tx_spec.outputs.push(OutputSpec::new(
                spent - (fee + spend),
                self.get_public_key().to_vec())
            )
        }
        
        tx_spec.to_transaction()
    }
}

#[derive(Serialize, Deserialize)]
struct SerdeWallet{
    pub utxos: HashMap<String, TxOutput>,
    pub funds: usize,
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

impl Serialize for Wallet{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let serde_utxos: HashMap<String, TxOutput> = self.utxos.iter().map(
            |((hash, index), output)|
            (format!("{}:{}", hex::encode(hash.clone()), index.clone()), output.clone())
        ).collect();
        
        let serde_wallet = SerdeWallet{
            utxos: serde_utxos,
            funds: self.funds,
            public_key: self.public_key.clone(),
            private_key: self.private_key.clone()
        };

        serde_wallet.serialize(serializer)
    }  
}

impl <'de>Deserialize<'de> for Wallet{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let serde_wallet = SerdeWallet::deserialize(deserializer)?;
        
        let utxos = serde_wallet.utxos.iter().map(
            |(key, output)|{

            let (hash, index) = key.split_once(":").expect("No colon found for utxos key"); 
            let hash = hex::decode(&hash).map_err(serde::de::Error::custom)?;
            let index: usize = index.parse().expect("index not a number");
            Ok(((hash, index.clone()), output.clone()))
            }
            
        ).collect::<Result<HashMap<_, _>, _>>()?;


        Ok(Wallet::from_serde_wallet(&serde_wallet, utxos))
    }
}