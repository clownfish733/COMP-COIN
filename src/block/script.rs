use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Script(Vec<OpCode>);

use crate::utils::sha256;

use super::{
    transaction::{Transaction, TxOutput},
    keys::PublicKey,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
enum OpCode{
    PUSHBYTES(Vec<u8>),
    DUP,
    SHA256,
    CHECKSIG,
    EQUALVERIFY
}

pub fn compute_sig_hash(
    tx: &Transaction,
    input_index: usize,
    utxo: &TxOutput,
) -> Vec<u8>{
    let mut modified_tx = tx.clone();
    for input in &mut modified_tx.inputs{
        input.unlocking_script = Script::empty();
    }

    modified_tx.inputs[input_index].unlocking_script = utxo.locking_script.clone();
    sha256(modified_tx.to_bytes())
}

impl Script{
    pub fn empty() -> Self{
        Self(vec![])
    }

    pub fn validate(
        &self,
        tx: &Transaction,
        input_index: usize,
        utxo: &TxOutput
    ) -> bool{
        let mut stack: Vec<Vec<u8>> = Vec::new();
        for op in self.0.iter(){
            match op{
                OpCode::PUSHBYTES(data) => {
                    stack.push(data.clone());
                }

                OpCode::DUP => {
                    if let Some(top) = stack.last(){
                        stack.push(top.clone())
                    }else{
                        return false
                    }
                }

                OpCode::SHA256 => {
                    if let Some(top) = stack.pop(){
                        stack.push(sha256(top.clone()));
                    }else{
                        return false
                    }
                }

                OpCode::EQUALVERIFY => {
                    if let Some(x1) = stack.pop() && let Some(x2) = stack.pop() {
                        if x1 != x2{
                            return false
                        }
                    }
                    else{
                        return false
                    }
                }

                OpCode::CHECKSIG => {
                    let sig_hash = compute_sig_hash(&tx, input_index, &utxo);

                    let Some(bytes) = stack.pop() else{
                        return false
                    };

                    let Ok(public_key) = PublicKey::from_bytes(bytes) else {
                        return false
                    };

                    let Some(signature) = stack.pop() else {
                        return false
                    };
                    match public_key.verify_sig(sig_hash, signature){
                        true => stack.push(vec![1]),
                        false => {
                            info!("1");
                            return false}
                    }
                }

            }
        }

        match stack.last(){
            Some(top) => top.iter().any(|&b| b != 0),
            None => {warn!("6"); return false},
        }
    }

    pub fn concat(s1: &Script, s2: &Script) -> Self{
        Self(vec![s1.0.clone(), s2.0.clone()].concat())
    }

    pub fn P2PKHUnlocking(sig: Vec<u8>, public_key: Vec<u8>) -> Self{
        Self(vec![
            OpCode::PUSHBYTES(sig),
            OpCode::PUSHBYTES(public_key)
        ])
    }

    pub fn P2PKHLocking(public_key_hash: Vec<u8>) -> Self{
        Self(vec![
            OpCode::DUP,
            OpCode::SHA256,
            OpCode::PUSHBYTES(public_key_hash),
            OpCode::EQUALVERIFY,
            OpCode::CHECKSIG,
        ])
    }

    pub fn P2PKHLocking_get_public_key_hash(&self) -> Option<Vec<u8>>{
        let Some(OpCode::PUSHBYTES(hash)) = self.0.get(2) else{
            return None
        };

        Some(hash.clone())
    }
}