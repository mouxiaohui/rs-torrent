use std::io::Read;

use anyhow::Result;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::bencode::de;

#[derive(Debug, Serialize, Deserialize)]
struct TorrentFile {
    name: String,
    info: Info,
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    pieces: ByteBuf,
}

pub fn parse_torrent_file(path: &str) -> Result<()> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let torrent_file: TorrentFile = de::from_bytes(&buf)?;
    println!("{:?}", torrent_file);

    Ok(())
}
