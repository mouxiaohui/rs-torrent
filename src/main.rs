mod bencode;
mod torrent;
use anyhow::Result;

fn main() -> Result<()> {
    torrent::parse_torrent_file("./temp/debian-iso.torrent")?;
    Ok(())
}
