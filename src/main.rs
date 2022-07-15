use anyhow::Result;
use bencode::bencode::BObject;

mod bencode;

fn main() -> Result<()> {
    let mut data: &[u8] = b"i-300e";
    let obj = BObject::parse(&mut data)?;
    println!("{:?}", obj);
    Ok(())
}
