use std::sync::{
    Arc,
    atomic::AtomicBool
};

use log::info;

use tokio::{
    sync::{RwLock, mpsc}
};

use anyhow::Result;

use crate::node::{
    Node,
    NetworkCommand
};

pub async fn start_ui_server(
    node: Arc<RwLock<Node>>,
    mut network_tx: mpsc::Sender<NetworkCommand>,
    save: Arc<AtomicBool>,
) -> Result<()>{

    info!("Started UI Server");
    loop{
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
    