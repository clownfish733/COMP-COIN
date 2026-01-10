use rand::{RngCore};


pub fn generate_nonce() -> Vec<u8>{
    let mut nonce = [0u8; 32];
    rand::rng().fill_bytes(&mut nonce);
    nonce.to_vec()
}