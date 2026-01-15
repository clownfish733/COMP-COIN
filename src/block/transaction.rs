use log::info;
use serde::{Serialize, Deserialize}; 

use crate::{utils::{get_timestamp, sha256}};

use super::{
    script::Script,
    keys::{PublicKey,PrivateKey},
    script::compute_sig_hash,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Transaction{
    timestamp: usize,
    version: usize,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>
}

impl Transaction{
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
                locking_script: Script::P2PKHLocking(public_key.to_hash()),
                value: reward
            }]
        }
    }

    pub fn add_fee(&mut self, public_key: PublicKey, fee: usize){
        self.outputs.insert(0, TxOutput { 
            value: fee, 
            locking_script: Script::P2PKHLocking(public_key.to_hash()) 
        })
    }

    pub fn remove_fee(&mut self){
        self.outputs.remove(0);
    }

    pub fn debug(&self){
        info!("Transaction");
        info!("Inputs:");
        for input in self.inputs.clone(){
            info!("\t{:?}", input);
        }
        info!("Outputs:");
        for output in self.outputs.clone(){
            info!("\t{:?}", output);
        }
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
    pub recipient: Vec<u8>,
}

impl OutputSpec{
    pub fn to_tx_output(&self) -> TxOutput{
        TxOutput { 
            value: self.value, 
            locking_script: Script::P2PKHLocking(
                sha256(self.recipient.clone())
            ) 
        }
    }

    pub fn new(value: usize, recipient: Vec<u8>) -> Self{
        Self { 
            value, 
            recipient 
        }
    }
}

pub struct InputSpec{
    prev: Vec<u8>,
    output_index: usize,
    pub utxo: TxOutput,
}

impl InputSpec{
    pub fn add_sig(
        &self, 
        public_key: Vec<u8>, 
        private_key: PrivateKey, 
        tx: Transaction, 
        index: usize
    ) -> TxInput{
        TxInput { 
            prev: self.prev.clone(), 
            output_index: self.output_index, 
            unlocking_script: Script::P2PKHUnlocking(
                private_key.sign(
                    compute_sig_hash(
                        &tx, 
                        index, 
                        &self.utxo
                    )),
                 public_key
                ) 
        }
    }

    fn to_sig_tx_input(&self) -> TxInput{
        TxInput {
            prev: self.prev.clone(),
            output_index: self.output_index, 
            unlocking_script: Script::empty() 
        }
    }

    pub fn new(
        prev: Vec<u8>,
        output_index: usize,
        utxo: TxOutput
    ) -> Self{
        Self { 
            prev, 
            output_index, 
            utxo
        }
    }
}

pub struct TransactionSpec{
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    pub version: usize,
}




impl TransactionSpec{
    pub fn to_transaction(&self) -> Transaction{
        let mut transaction = self.to_sig_transaction();
        
        for index in 0..transaction.inputs.len(){
            transaction.inputs[index].unlocking_script = Script::P2PKHUnlocking(
                self.private_key.sign(
                    compute_sig_hash(
                        &transaction.clone(), 
                        index, 
                        &self.inputs.get(index).unwrap().utxo
                    )),
                 self.public_key.to_vec()
                ) 
        }
        
        transaction
    }

    fn to_sig_transaction(&self) -> Transaction{
        Transaction { 
            timestamp: get_timestamp(), 

            version: self.version, 

            inputs: self.inputs.iter().map(
                |input| 
                input.to_sig_tx_input()
            ).collect(),

            outputs: self.outputs.iter().map(
                |output| 
                output.to_tx_output()
            ).collect()
        }
    }

    pub fn pre_inputs(
        version: usize, 
        outputs: Vec<OutputSpec>, 
        public_key: PublicKey, 
        private_key: PrivateKey
    ) -> Self{
        Self {
            public_key, 
            private_key, 
            inputs: vec![], 
            outputs, 
            version 
        }
    }
}