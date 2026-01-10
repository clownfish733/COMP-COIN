use serde::{Serialize, Deserialize}; 

#[derive(Serialize, Deserialize, Clone)]
pub struct Mempool;

impl Mempool {
    pub fn new() -> Self{
        Self
    }
}