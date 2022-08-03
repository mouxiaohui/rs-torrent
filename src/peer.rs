use std::net::Ipv4Addr;

use anyhow::{Result, anyhow};
use serde_bytes::ByteBuf;

#[derive(Debug)]
pub struct PeerInfo {
    pub ip: Ipv4Addr,
    pub port: u16,
}

pub async fn find_peers_info(peers: ByteBuf) -> Result<Vec<PeerInfo>>{
    if peers.len() % 6 != 0 {
        return Err(anyhow!("Received malformed peers"));
    }

    let mut peers_info = Vec::new();
    for i in 0..(peers.len() / 6) {
        let offset = i * 6;
        let ip = &peers[offset..(offset + 4)];
        let port = &peers[(offset + 4)..(offset + 6)];

        peers_info.push(PeerInfo {
            ip: Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]),
            port: u16::from_be_bytes(port.try_into()?),
        });
    }

    Ok(peers_info)
}

// pub async fn handshake(client: &mut TcpStream,peer_info: &PeerInfo, peer_id: &[u8]) -> Result<()>{
//     Ok(())
// }
