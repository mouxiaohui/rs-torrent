use anyhow::Result;

pub struct Handshake<'a> {
    pstr: &'a str,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl<'a> Handshake<'a> {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            pstr: "BitTorrent protocol",
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.to_vec(),
        }
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![self.pstr.len().try_into()?];
        buf.append(&mut self.pstr.as_bytes().to_vec());
        buf.append(&mut vec![0; 8]);
        buf.append(&mut self.info_hash);
        buf.append(&mut self.peer_id);

        Ok(buf)
    }
}
