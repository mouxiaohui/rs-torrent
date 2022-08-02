use std::io::Read;

use anyhow::Result;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::Url;
use serde::{Deserializer, Serializer};
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::{
    bencode::{de, ser},
    utils::{self, random_array},
};

fn de_with_piece<'de, D>(de: D) -> Result<Vec<[u8; 20]>, D::Error>
where
    D: Deserializer<'de>,
{
    let data: ByteBuf = serde::de::Deserialize::deserialize(de)?;
    let mut res = Vec::new();

    let dual = data.len() % 20;
    let mut i = 0;
    let mut container = [0u8; 20];
    for d in data {
        container[i] = d;
        i += 1;
        if i == 20 {
            i = 0;
            res.push(container);
        }
    }
    if dual != 0 {
        for i in dual..20 {
            container[i] = 0;
        }
        res.push(container)
    }

    Ok(res)
}

fn ser_with_piece<S>(piece: &Vec<[u8; 20]>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_bytes(&piece.concat())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    announce: String,
    info: Info,
    #[serde(skip)]
    info_hash: [u8; 20],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    #[serde(deserialize_with = "de_with_piece", serialize_with = "ser_with_piece")]
    pieces: Vec<[u8; 20]>,
    #[serde(rename(serialize = "piece length", deserialize = "piece length"))]
    piece_length: u64,
    length: u64,
    name: String,
}

pub fn parse_torrent_file(path: &str) -> Result<TorrentFile> {
    let mut buf = Vec::new();
    let mut file = std::fs::File::open(path)?;

    file.read_to_end(&mut buf)?;
    let mut tf: TorrentFile = de::from_bytes(&buf)?;
    tf.info_hash = utils::sha_from(ser::to_bytes(&tf.info)?);

    Ok(tf)
}

pub fn build_client(tf: &TorrentFile) -> Result<Url> {
    let mut url = Url::parse(&tf.announce)?;
    let mut peer_id = [0u8; 20];
    random_array(&mut peer_id);

    let peer_id = percent_encode(&peer_id, NON_ALPHANUMERIC).to_string();
    let hash_encode = percent_encode(&tf.info_hash, NON_ALPHANUMERIC).to_string();
    let length_string = tf.info.length.to_string();

    let qs = vec![
        ("info_hash", hash_encode.as_ref()),
        ("peer_id", peer_id.as_ref()),
        ("port", "6881"),
        ("uploaded", "0"),
        ("downloaded", "0"),
        ("compact", "1"),
        ("left", length_string.as_ref()),
    ];
    url.set_query(Some(&set_querys(qs)));

    Ok(url)
}

fn set_querys(qs: Vec<(&str, &str)>) -> String {
    let querys: Vec<String> = qs.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    querys.join("&")
}
