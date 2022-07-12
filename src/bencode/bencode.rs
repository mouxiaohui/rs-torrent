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

        len = (len * 10) + s.parse::<usize>().map_err(|_| BencodeError::TypeError)?;
    }

    let mut buf = vec![0; len];
    reader.read(&mut buf)?;
    let res = std::str::from_utf8(&buf)?;

    Ok(res.to_string())
}

pub fn encode_int<W: Write>(writer: &mut W, val: i64) -> Result<usize> {
    let val_fmt = format!("i{}e", val);
    let size = writer.write(val_fmt.as_bytes())?;

    Ok(size)
}

pub fn decode_int<R: Read>(reader: &mut R) -> Result<i64> {
    let mut buf = [0; 1];

    reader.read(&mut buf)?;
    if buf[0] as char != 'i' {
        return Err(BencodeError::TypeError.into());
    }

    let mut num = 0;
    loop {
        reader.read(&mut buf)?;
        let s = std::str::from_utf8(&buf)?;
        if s == "e" {
            break;
        }

        num = (num * 10) + s.parse::<i64>().map_err(|_| BencodeError::TypeError)?;
    }

    Ok(num)
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

    #[test]
    fn test_encode_int() {
        let mut writer = vec![];
        encode_int(&mut writer, 8848).unwrap();
        assert_eq!("i8848e", std::str::from_utf8(&writer).unwrap());
    }

    #[test]
    fn test_decode_int() {
        let mut reader: &[u8] = b"i8848e";
        let num = decode_int(&mut reader).unwrap();
        assert_eq!(8848, num);
    }
}
