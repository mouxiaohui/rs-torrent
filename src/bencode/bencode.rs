use std::{
    collections::HashMap,
    io::{Read, Write},
};

use anyhow::Result;

use super::err::BencodeError;

enum BObject {
    BStr(String),
    BInt(i64),
    BList(Vec<Box<BObject>>),
    BDict(HashMap<String, Box<BObject>>),
}

impl BObject {
    // pub fn new(b: u8) -> Result<Self> {
    //     match b {
    //         0x01 => Ok(BType::BSTR),
    //         0x02 => Ok(BType::BINT),
    //         0x03 => Ok(BType::BLIST),
    //         0x04 => Ok(BType::BDICT),
    //         _ => Err(BencodeError::TypeError.into()),
    //     }
    // }
}

pub fn encode_string<W: Write>(writer: &mut W, val: String) -> Result<usize> {
    let val_fmt = format!("{}:{}", val.len(), val);
    let size = writer.write(val_fmt.as_bytes())?;
    writer.flush()?;

    Ok(size)
}

pub fn decode_string<R: Read>(reader: &mut R) -> Result<String> {
    let mut buf = [0; 1];

    let mut len = 0;
    loop {
        reader.read(&mut buf)?;
        let s = std::str::from_utf8(&buf)?;
        if s == ":" {
            break;
        }

        len = (len * 10)
            + s.parse::<usize>()
                .map_err(|_| BencodeError::InvalidBencode)?;
    }

    let mut buf = vec![0; len];
    reader.read(&mut buf)?;
    let res = std::str::from_utf8(&buf)?;

    Ok(res.to_string())
}

mod test {
    use super::*;

    #[test]
    fn test_encode_string() {
        let mut writer = vec![];
        encode_string(&mut writer, "hello".to_string()).unwrap();
        assert_eq!("5:hello", std::str::from_utf8(&writer).unwrap());
    }

    #[test]
    fn test_decode_string() {
        let mut reader: &[u8] = b"5:hello";
        let res = decode_string(&mut reader).unwrap();
        assert_eq!("hello".to_string(), res);
    }
}
