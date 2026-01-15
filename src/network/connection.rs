use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, 
    net::tcp::{OwnedReadHalf, OwnedWriteHalf}, 
    sync::mpsc
};

use anyhow::Result;

use log::{error, info};

#[derive(Clone)]
pub struct ConnectionEvent{
    pub network_address: SocketAddr,
    pub event: ConnectionType,
}

impl ConnectionEvent{
    fn new(
        network_address: SocketAddr, 
        event: ConnectionType
    ) -> Self{
        Self { 
            network_address, 
            event 
        }
    }

    pub fn close(network_address: SocketAddr) -> Self{
        Self::new(
            network_address,
            ConnectionType::Close,
        )
    }
    pub fn message(
        network_address: SocketAddr, 
        msg: Vec<u8>
    ) -> Self{
        Self::new(
            network_address,
            ConnectionType::Message(msg)
        )   
    }
}

#[derive(Clone)]
pub enum ConnectionType{
    Close,
    Message(Vec<u8>),
}

#[derive(Clone)]
pub struct ConnectionResponse{
    response: ConnectionType,
}

impl ConnectionResponse{
    fn new(response: ConnectionType) -> Self{
        Self { 
            response 
        }
    }

    pub fn close() -> Self{
        Self::new(
            ConnectionType::Close,
        )
    }

    pub fn message(msg: Vec<u8>) -> Self{
        Self::new(
            ConnectionType::Message(msg)
        )
    }
}


pub async fn connection_receiver(
    mut reader: OwnedReadHalf,
    peer: SocketAddr,
    event_tx: mpsc::Sender<ConnectionEvent>
) -> Result<()>{
    loop{    
        let mut len_bytes = [0u8; 4];

        match reader.read_exact(&mut len_bytes).await{
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                event_tx.send(ConnectionEvent::close(peer.clone())).await?;
                return Ok(())
            }
            Err(e) => {
                error!("Error reading from: {}", peer);
                event_tx.send(ConnectionEvent::close(peer.clone())).await?;
                return Err(e.into())
            }
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;

        info!("Received message from: {}", &peer);

        event_tx.send(ConnectionEvent::message(peer.clone(), buf.clone())).await?;

    }
}


pub async fn connection_sender(
    mut writer: OwnedWriteHalf,
    mut response_rx: mpsc::Receiver<ConnectionResponse>,
) -> Result<()>{
    while let Some(response) = response_rx.recv().await{
        match response.response{
            ConnectionType::Close => {
                writer.shutdown().await?;
                return Ok(())
            }

            ConnectionType::Message(msg) => {
                let len = (msg.len() as u32).to_be_bytes();
                writer.write_all(&len).await?;
                writer.write_all(&msg).await?;
            }
        }
    }

    Ok(())
}