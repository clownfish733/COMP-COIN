use std::collections::HashMap;

use super::{
    transaction::{TxOutput, Transaction},
    keys::{PublicKey,PrivateKey},
    transaction::TransactionSpec,
};

pub struct Wallet{
    utxos: HashMap<(Vec<u8>, usize), TxOutput>,
    public_key: PublicKey,
    private_key: PrivateKey
}

impl Wallet{
    pub fn new() -> Self{
        let private_key = PrivateKey::new();
        let public_key = private_key.get_public_key();

        Self{
            utxos: HashMap::new(),
            public_key,
            private_key,
        }
    }

    fn insert(&mut self, hash: Vec<u8>, index: usize, utxo: TxOutput){
        self.utxos.insert((hash, index), utxo);
    }

    fn remove(&mut self, hash: Vec<u8>, index: usize){
        self.utxos.remove(&(hash, index));
    }

    fn add_transaction(&mut self, tx: Transaction, public_key: PublicKey){
        let hash = tx.get_hash();

        for input in tx.inputs{
            self.remove(input.prev, input.output_index);
        }

        for (index, output) in tx.outputs.iter().enumerate(){
            if output.locking_script.P2PKHLocking_get_public_key_hash().unwrap() 
                == self.public_key.get_public_key_hash(){
                self.insert(hash.clone(), index, output.clone());
            }
        }
    }

    pub fn new_transaction(tx_spec: TransactionSpec, fee: usize) -> Transaction{
            let amount_in = tx_spec.inputs.iter().map(|input| input.value)
    }
}