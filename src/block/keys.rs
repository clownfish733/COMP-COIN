use k256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::Verifier};

use anyhow::Result;
use rand_core::OsRng;

use crate::utils::sha256;

pub struct PublicKey(VerifyingKey);

pub struct PrivateKey(SigningKey);

impl PrivateKey{
    pub fn new() -> Self{
        Self(SigningKey::random(&mut OsRng))
    }

    pub fn get_public_key(&self) -> PublicKey{
        PublicKey(VerifyingKey::from(&self.0))
    }
}

impl PublicKey{
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self>{
        Ok(Self(VerifyingKey::from_sec1_bytes(&bytes)?))
    }

    pub fn verify_sig(&self, sig_hash: Vec<u8>, signature: Vec<u8>) -> bool{
        let Ok(signature) = Signature::from_slice(&signature)else{
            return false
        };
        self.0.verify(&sig_hash, &signature).is_ok()
    }

    pub fn get_public_key(&self) -> Vec<u8>{
        self.0.to_sec1_bytes().to_vec()
    }

    pub fn get_public_key_hash(&self) -> Vec<u8>{
        sha256(self.get_public_key())
    }
}
