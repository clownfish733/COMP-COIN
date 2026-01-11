use serde::{Serialize, Deserialize}; 

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction;

impl Transaction{
    pub fn is_valid(&self) -> bool{
        return true
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        postcard::to_allocvec(self).expect("Faile to serialize transaction")
    }
}