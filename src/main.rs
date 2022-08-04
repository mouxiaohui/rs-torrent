use anyhow::Result;

mod bencode;
mod download;
mod peer;
mod torrent_file;
mod tracker;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    download::download_to_file( "./temp/debian-iso.torrent", "out_path").await
}

