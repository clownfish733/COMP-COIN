use serde::{Serialize, Deserialize}; 

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction;

impl Transaction{
    pub fn is_valid(&self) -> bool{
        return true
    }
}