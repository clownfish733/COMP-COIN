use serde::{Serialize, Deserialize}; 

use crate::utils::{get_timestamp, sha256};

use super::{
    script::Script,
    keys::PublicKey,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Transaction{
    timestamp: usize,
    version: usize,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>
}

impl Transaction{
    pub fn is_valid(&self) -> bool{
        return true
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        postcard::to_allocvec(self).expect("Failed to serialize transaction")
    }

    pub fn get_hash(&self) -> Vec<u8>{
        sha256(self.to_bytes())
    }

    pub fn reward(reward: usize, public_key: PublicKey, version: usize) -> Self{
        Self{
            timestamp: get_timestamp(),
            version,
            inputs: vec![],
            outputs: vec![TxOutput{
                locking_script: Script::P2PKHLocking(public_key.get_public_key_hash()),
                value: reward
            }]
        }
    }

    pub fn add_fee(&mut self, public_key: PublicKey, fee: usize){
        self.outputs.push(TxOutput { 
            value: fee, 
            locking_script: Script::P2PKHLocking(public_key.get_public_key_hash()) 
        })
    }

    pub fn remove_fee(&mut self){
        self.outputs.pop();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TxInput{
    pub prev: Vec<u8>,
    pub output_index: usize,
    pub unlocking_script: Script,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TxOutput{
    pub value: usize,
    pub locking_script: Script,
}


pub struct OutputSpec{
    pub value: usize,
    pub receipient: Vec<u8>,
}

impl OutputSpec{
    pub fn to_tx_output(&self) -> TxOutput{
        TxOutput { 
            value: self.value, 
            locking_script: Script::P2PKHLocking(
                sha256(self.receipient.clone())
            ) 
        }
    }
}

pub struct InputSpec{
    prev: Vec<u8>,
    output_index: usize,
    sig: Vec<u8>,
}

impl InputSpec{
    pub fn to_tx_input(&self, public_key: Vec<u8>) -> TxInput{
        TxInput { 
            prev: self.prev.clone(), 
            output_index: self.output_index, 
            unlocking_script: Script::P2PKHUnlocking(self.sig.clone(), public_key) 
        }
    }
}

pub struct TransactionSpec{
    pub public_key: PublicKey,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    pub version: usize,
}




impl TransactionSpec{
    pub fn to_transaction(&self) -> Transaction{
        Transaction { 
            timestamp: get_timestamp(),

            version: self.version,

            inputs: self.inputs.iter().map(
                |input| 
                input.to_tx_input(self.public_key.get_public_key()))
                .collect(),

            outputs: self.outputs.iter().map(
                |output|
                output.to_tx_output()
            )
            .collect()
        }
    }
}