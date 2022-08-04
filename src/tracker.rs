use anyhow::Result;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::Url;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::bencode::de;
use crate::torrent_file::TorrentFile;

const PORT: &str = "6881";

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerResp {
    pub interval: u64,
    pub peers: ByteBuf,
}

pub async fn request_tracker(tf: &TorrentFile, peer_id: &[u8]) -> Result<TrackerResp> {
    let url = build_url(tf, peer_id)?;
    let client = reqwest::get(url).await?;

    Ok(de::from_bytes(&client.bytes().await?)?)
}

fn build_url(tf: &TorrentFile, peer_id: &[u8]) -> Result<Url> {
    let mut url = Url::parse(&tf.announce)?;
    let peer_id = percent_encode(peer_id, NON_ALPHANUMERIC).to_string();
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
