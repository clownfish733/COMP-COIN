use COMP_COIN::block::{UTXOS, Block};


fn main() {
    let block = Block::temp_block();
    let mut utxos = UTXOS::new();
    utxos.add_block(&block);
    match serde_json::to_string_pretty(&utxos){
        Ok(json) => {
            println!("{}", json);
        }
        Err(e) => {
            println!("{}",e);
        }
    }
}