use anyhow::Result;

use bencode::bencode::encode_string;

use crate::bencode::bencode::decode_string;

mod bencode;

fn main() -> Result<()> {
    let mut data = vec![];
    encode_string(&mut data, "hello".to_string())?;
    println!("{:?}", data);

    let mut reader: &[u8] = b"5:hello";
    println!("{:?}", reader);
    let res = decode_string(&mut reader)?;
    println!("result = {}", res);

    Ok(())
}
