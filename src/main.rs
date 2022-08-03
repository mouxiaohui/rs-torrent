use anyhow::Result;

mod bencode;
mod torrent;
mod tracker;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let tf = torrent::parse_torrent_file("./temp/debian-iso.torrent")?;

    Ok(())
}
