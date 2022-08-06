use std::time::Duration;

use anyhow::Result;
use tokio::io::AsyncWriteExt;

use crate::handshake::Handshake;
use crate::peer::{find_peers, Peer};
use crate::torrent_file::{self, TorrentFile};
use crate::tracker::request_tracker;
use crate::utils::random_array;

struct Torrent {
    peer_id: [u8; 20],
    peers: Vec<Peer>,
    info_hash: [u8; 20],
    piece_hashes: Vec<[u8; 20]>,
    piece_lenght: usize,
    lenght: usize,
    name: String,
}

impl Torrent {
    async fn new(tf: TorrentFile) -> Result<Self> {
        let mut peer_id = [0u8; 20];
        random_array(&mut peer_id);

        let resp = request_tracker(&tf, &peer_id).await?;
        let peers = find_peers(resp.peers).await?;

        Ok(Self {
            peer_id,
            peers,
            info_hash: tf.info_hash,
            piece_hashes: tf.info.pieces,
            piece_lenght: tf.info.piece_length,
            lenght: tf.info.length,
            name: tf.info.name,
        })
    }

    fn bounds_for_piece(&self, index: usize) -> (usize, usize) {
        let begin = index * self.piece_lenght;
        let mut end = begin + self.piece_lenght;
        if end > self.lenght {
            end = self.lenght
        }

        (begin, end)
    }

    async fn download(&self) -> Result<()> {
        println!("Start download for {}", self.name);

        for peer in &self.peers {
            let mut stream = peer.connect(Duration::from_secs(30)).await?;
            let mut handshake = Handshake::new(self.info_hash, self.peer_id);
            stream.write_all(&handshake.serialize()?).await?;
        }

        Ok(())
    }
}

pub async fn download_to_file(file_path: &str, _out_path: &str) -> Result<()> {
    let tf = torrent_file::parse_torrent_file(file_path)?;
    let torrent = Torrent::new(tf).await?;
    torrent.download().await?;

    Ok(())
}
