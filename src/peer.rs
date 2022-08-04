use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use anyhow::{anyhow, Result};
use serde_bytes::ByteBuf;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

impl Peer {
    const IP_LEN: usize = 4;
    const PORT_LEN: usize = 2;
    const LENGTH: usize = Self::IP_LEN + Self::PORT_LEN;

    fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    pub async fn connect(&self, timeout: Duration) -> Result<TcpStream> {
        match tokio::time::timeout(timeout, TcpStream::connect(self.addr())).await {
            Ok(c) => match c {
                Ok(ok) => Ok(ok),
                Err(e) => Err(anyhow!("Error while connecting to server: {}", e)),
            },
            Err(_) => return Err(anyhow!("Timeout while connecting to server")),
        }
    }
}

impl TryFrom<&[u8]> for Peer {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::LENGTH {
            return Err(anyhow!("Received malformed peers"));
        }

        Ok(Self {
            ip: IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(
                bytes[..Self::IP_LEN].try_into()?,
            ))),
            port: u16::from_be_bytes(bytes[Self::IP_LEN..].try_into()?),
        })
    }
}

pub async fn find_peers(peers: ByteBuf) -> Result<Vec<Peer>> {
    let peer_chunks: Vec<&[u8]> = peers.chunks(Peer::LENGTH).collect();
    let mut peers = Vec::new();
    for chunk in peer_chunks {
        peers.push(Peer::try_from(chunk)?);
    }

    Ok(peers)
}

