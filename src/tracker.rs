use std::net::Ipv4Addr;

use anyhow::{anyhow, Result};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::Url;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::bencode::de;
use crate::torrent::TorrentFile;
use crate::utils::random_array;

const PORT: &str = "6881";
const IP_LEN: usize = 4;
const PORT_LEN: usize = 2;

#[derive(Debug)]
pub struct PeerInfo {
    ip: Ipv4Addr,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackerResp {
    interval: u64,
    peers: ByteBuf,
}

pub async fn find_peers(tf: &TorrentFile) -> Result<Vec<PeerInfo>> {
    let url = build_url(tf)?;
    let client = reqwest::get(url).await?;
    let resp: TrackerResp = de::from_bytes(&client.bytes().await?)?;

    let peer_len = IP_LEN + PORT_LEN;
    if resp.peers.len() % peer_len != 0 {
        return Err(anyhow!("Received malformed peers"));
    }

    let mut peers_info = Vec::new();
    for i in 0..(resp.peers.len() / peer_len) {
        let offset = i * 6;
        let ip = &resp.peers[offset..(offset + IP_LEN)];
        let port = &resp.peers[(offset + IP_LEN)..(offset + peer_len)];

        peers_info.push(PeerInfo {
            ip: Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]),
            port: u16::from_be_bytes(port.try_into()?),
        });
    }

    Ok(peers_info)
}

fn build_url(tf: &TorrentFile) -> Result<Url> {
    let mut url = Url::parse(&tf.announce)?;
    let mut peer_id = [0u8; 20];
    random_array(&mut peer_id);

    let peer_id = percent_encode(&peer_id, NON_ALPHANUMERIC).to_string();
    let hash_encode = percent_encode(&tf.info_hash, NON_ALPHANUMERIC).to_string();
    let length_string = tf.info.length.to_string();

    let qs = vec![
        ("info_hash", hash_encode.as_ref()),
        ("peer_id", peer_id.as_ref()),
        ("port", PORT),
        ("uploaded", "0"),
        ("downloaded", "0"),
        ("compact", "1"),
        ("left", length_string.as_ref()),
    ];
    url.set_query(Some(&gen_querys(qs)));

    Ok(url)
}

fn gen_querys(qs: Vec<(&str, &str)>) -> String {
    let querys: Vec<String> = qs.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    querys.join("&")
}
