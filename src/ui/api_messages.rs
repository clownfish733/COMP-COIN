use log::warn;
use serde::{Serialize, Deserialize};

use std::{
    collections::HashMap,
    fs::File, path::PathBuf, 
};

use anyhow::Result;

use log::info;

use crate::block::OutputSpec;

const FILE_PATH: &str = "configs/AddressBook.json";

#[derive(Serialize)]
pub struct NodeStatus{
    height: usize,
    mempool_size: usize,
    difficulty: usize,
}

impl NodeStatus{
    pub fn new(
        height: usize,
        mempool_size: usize,
        difficulty: usize
    ) -> Self{
        Self{
            height,
            mempool_size,
            difficulty
        }
    }

}

#[derive(Serialize)]
pub struct UserStatus{
    amount: usize,
    pk: String
}

impl UserStatus{
    pub fn new(amount: usize, pk: String) -> Self{
        Self { 
            amount, 
            pk 
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionRequest{
    pub recipients: Vec<(String, usize)>,
    pub fee: usize,
}

impl TransactionRequest{
    pub fn log(&self){
        info!("New Transaction requested");
        info!("\tRecipients");
        for (recipient, value) in self.recipients.iter(){
            info!("\t\t{}:{}",recipient, value);
        }
        info!("\tFee");
        info!("{}", self.fee);
    }

    pub fn calculate_total_spend(&self) -> usize{
        self.recipients.iter().map(
            |(_r, value)| value
        ).sum::<usize>() 
        + self.fee
    }

    pub fn get_outputs(&self) -> Result<Vec<OutputSpec>> {
    self.recipients.iter().map(
        |(recipient, value)| {
            Ok(OutputSpec::new(*value, hex::decode(recipient)?))
        }
    ).collect()

    }
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse{
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddressBook(HashMap<String, String>);

impl AddressBook{
     fn new() -> Self{
        Self(HashMap::new())
    }
    pub fn load() -> Self{
        if let Ok(file) = File::open(FILE_PATH){
            match serde_json::from_reader(file){
                Ok(address_book) => address_book,
                Err(e) => {
                    warn!("Unable to read from addresss book: {}", e);
                    AddressBook::new()
                }
            }
        }else{
            warn!("Address book file could not be found");
            AddressBook::new()
        }
    }

    pub fn save(&self){
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FILE_PATH);
        let file = File::create(file_path).expect("Unable to open address book");
        serde_json::to_writer(file, self).expect("Unable to write to address book");

    }
}