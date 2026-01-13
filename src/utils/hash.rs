use sha2::{Sha256, Digest};

pub fn sha256(message: Vec<u8>) -> Vec<u8>{
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().to_vec()
}
