use std::net::SocketAddr;

#[derive(Clone)]
pub struct ConnectionEvent{
    network_address: SocketAddr,
    event: ConnectionType,
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

    fn close(network_address: SocketAddr) -> Self{
        Self::new(
            network_address,
            ConnectionType::Close,
        )
    }
    fn message(
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
enum ConnectionType{
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

    fn close() -> Self{
        Self::new(
            ConnectionType::Close,
        )
    }

    fn message(msg: Vec<u8>) -> Self{
        Self::new(
            ConnectionType::Message(msg)
        )
    }
}