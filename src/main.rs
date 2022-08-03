use anyhow::Result;
use peer::find_peers_info;
use tracker::request_tracker;
use utils::random_array;

mod bencode;
mod peer;
mod torrent;
mod tracker;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    download_to_file().await
}

async fn download_to_file() -> Result<()>{
    let mut peer_id = [0u8; 20];
    random_array(&mut peer_id);

    let tf = torrent::parse_torrent_file("./temp/debian-iso.torrent")?;
    let resp  = request_tracker(&tf, &peer_id).await?;
    let peers_info = find_peers_info(resp.peers).await?;
    println!("{:?}", peers_info);

    // let peer = peer_info[0];
    // TcpStream::connect(peer.);
    // handshake(&peer_info[0]).await?;

    Ok(())
}
