use serde::{Serialize, Deserialize}; 

use anyhow::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct Block;

impl Block{
    pub fn is_valid(&self) -> bool{
        return true
    }

    pub fn get_height(&self) -> usize{
        return 0
    }
}