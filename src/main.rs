mod bencode;
mod torrent;
mod utils;
use anyhow::Result;

use crate::torrent::build_client;

#[tokio::main]
async fn main() -> Result<()> {
    let tf = torrent::parse_torrent_file("./temp/debian-iso.torrent")?;
    let url = build_client(&tf)?;
    println!("{:?}", url.to_string());

    Ok(())
}
