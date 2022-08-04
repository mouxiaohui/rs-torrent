use std::io::Read;

use anyhow::Result;
use serde::{Deserializer, Serializer};
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::bencode::{de, ser};
use crate::utils::sha_from;

fn de_with_pieces<'de, D>(de: D) -> Result<Vec<[u8; 20]>, D::Error>
where
    D: Deserializer<'de>,
{
    let data: ByteBuf = serde::de::Deserialize::deserialize(de)?;

    Ok(data
        .chunks(20)
        .map(|f| f.try_into().expect("pieces with incorrect length"))
        .collect())
}

fn ser_with_pieces<S>(pieces: &Vec<[u8; 20]>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_bytes(&pieces.concat())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    pub announce: String,
    pub info: Info,
    #[serde(skip)]
    pub info_hash: [u8; 20],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    #[serde(
        deserialize_with = "de_with_pieces",
        serialize_with = "ser_with_pieces"
    )]
    pub pieces: Vec<[u8; 20]>,
    #[serde(rename(serialize = "piece length", deserialize = "piece length"))]
    pub piece_length: usize,
    pub length: usize,
    pub name: String,
}

pub fn parse_torrent_file(path: &str) -> Result<TorrentFile> {
    let mut buf = Vec::new();
    let mut file = std::fs::File::open(path)?;

    file.read_to_end(&mut buf)?;
    let mut tf: TorrentFile = de::from_bytes(&buf)?;
    tf.info_hash = sha_from(ser::to_bytes(&tf.info)?);

    Ok(tf)
}
