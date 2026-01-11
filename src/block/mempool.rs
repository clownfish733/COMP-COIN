use serde::{Serialize, Deserialize}; 

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mempool;

impl Mempool {
    pub fn new() -> Self{
        Self
    }
}