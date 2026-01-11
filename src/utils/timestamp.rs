use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> usize{
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize
}