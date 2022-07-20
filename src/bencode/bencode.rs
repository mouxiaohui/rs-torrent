use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use itoa::Integer;

use super::err::Error;

fn parse_utf8<F>(v: &[u8]) -> Result<F>
where
    F: FromStr,
{
    match std::str::from_utf8(v)?.parse::<F>() {
        Ok(res) => Ok(res),
        Err(_) => Err(anyhow!("parse failure: {:?}", v)),
    }
}

fn peek(reader: &mut dyn BufRead) -> Result<u8> {
    let peek = reader.fill_buf()?;
    if peek.len() == 0 {
        return Err(anyhow!("end error"));
    }
    Ok(peek[0])
}

#[derive(Debug, PartialEq)]
pub enum Num {
    U64(u64),
    I64(i64),
}

#[derive(Debug, PartialEq)]
pub struct Map<K, V> {
    pub k: VecDeque<K>,
    pub v: VecDeque<V>,
}

impl<K, V> Map<K, V> {
    fn new() -> Self {
        Self {
            k: VecDeque::new(),
            v: VecDeque::new(),
        }
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for Map<K, V> {
    fn from(arr: [(K, V); N]) -> Self {
        let mut m = Self::new();
        for (k, v) in arr {
            m.k.push_back(k);
            m.v.push_back(v);
        }

        m
    }
}

#[derive(Debug, PartialEq)]
pub enum BObject {
    BStr(String),
    BInt(Num),
    BList(VecDeque<BObject>),
    BDict(Map<String, BObject>),
}

impl BObject {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        parse_bencode(&mut reader)
    }
}

fn decode_string_len<R: ?Sized>(reader: &mut R) -> Result<usize>
where
    R: Read,
{
    let mut buf = [0; 1];

    let mut len = 0;
    loop {
        reader.read(&mut buf)?;
        if buf[0] as char == ':' {
            break;
        }

        match parse_utf8::<usize>(&buf) {
            Ok(v) => len = (len * 10) + v,
            Err(_) => return Err(anyhow!(Error::InvalidStr)),
        };
    }

    Ok(len)
}

fn decode_string<R: ?Sized>(reader: &mut R) -> Result<String>
where
    R: Read,
{
    let len = decode_string_len(reader)?;
    let mut bytes = vec![0; len];
    reader.read(&mut bytes)?;
    let res = std::str::from_utf8(&bytes)?;

    Ok(res.to_string())
}

fn decode_int_bytes<R: ?Sized>(reader: &mut R) -> Result<Vec<u8>>
where
    R: Read,
{
    let mut buf = [0; 1];
    reader.read(&mut buf)?;
    if buf[0] != b'i' {
        return Err(anyhow!(Error::InvalidInteger));
    }

    let mut bytes = Vec::new();
    loop {
        reader.read(&mut buf)?;
        if buf[0] == b'e' {
            break;
        }

        bytes.push(buf[0]);
    }

    Ok(bytes)
}

fn decode_int<R: ?Sized>(reader: &mut R) -> Result<Num>
where
    R: Read,
{
    let mut bytes = decode_int_bytes(reader)?;
    if bytes[0] == b'-' {
        return Ok(Num::I64(parse_utf8(&mut bytes)?));
    }

    Ok(Num::U64(parse_utf8(&mut bytes)?))
}

pub fn parse_bencode_bytes<R: BufRead>(reader: &mut R) -> Result<Vec<u8>> {
    match peek(reader)? {
        b'0'..=b'9' => {
            let len = decode_string_len(reader)?;
            let mut bytes = vec![0; len];
            reader.read(&mut bytes)?;
            Ok(bytes)
        }
        b'i' => decode_int_bytes(reader),
        b'l' => {
            reader.consume(1);
            let mut bytes = Vec::new();
            loop {
                if peek(reader)? == b'e' {
                    reader.consume(1);
                    break;
                }
                bytes.append(&mut parse_bencode_bytes(reader)?);
            }

            Ok(bytes)
        }
        b'd' => {
            reader.consume(1);
            let mut bytes = Vec::new();
            loop {
                if peek(reader)? == b'e' {
                    reader.consume(1);
                    break;
                }

                let mut key_bytes = parse_bencode_bytes(reader)?;
                let mut val_bytes = parse_bencode_bytes(reader)?;
                bytes.append(&mut key_bytes);
                bytes.append(&mut val_bytes);
            }

            Ok(bytes)
        }
        n @ _ => {
            let c = n as char;
            Err(anyhow!(Error::InvalidType(c.to_string())))
        }
    }
}

fn parse_bencode<R: BufRead>(reader: &mut R) -> Result<BObject> {
    match peek(reader)? {
        b'0'..=b'9' => Ok(BObject::BStr(decode_string(reader)?)),
        b'i' => Ok(BObject::BInt(decode_int(reader)?)),
        b'l' => {
            reader.consume(1);
            let mut list = VecDeque::new();
            loop {
                if peek(reader)? == b'e' {
                    reader.consume(1);
                    break;
                }
                list.push_back(parse_bencode(reader)?);
            }

            Ok(BObject::BList(list))
        }
        b'd' => {
            reader.consume(1);
            let mut dict = Map::new();
            loop {
                if peek(reader)? == b'e' {
                    reader.consume(1);
                    break;
                }
                let key = decode_string(reader)?;
                let val = parse_bencode(reader)?;
                dict.k.push_back(key);
                dict.v.push_back(val);
            }

            Ok(BObject::BDict(dict))
        }
        n @ _ => {
            let c = n as char;
            Err(anyhow!(Error::InvalidType(c.to_string())))
        }
    }
}

pub fn encode_int<W, I>(writer: &mut W, i: I) -> Result<()>
where
    I: Integer,
    W: Write,
{
    writer.write_all(b"i")?;
    writer.write_all(itoa::Buffer::new().format(i).as_bytes())?;
    writer.write_all(b"e")?;

    Ok(())
}

pub fn encode_bytes<W>(writer: &mut W, v: &[u8]) -> Result<()> where W: Write{
    writer.write_all(itoa::Buffer::new().format(v.len()).as_bytes())?;
    writer.write_all(b":")?;
    writer.write_all(v)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode_string() {
        let mut reader: &[u8] = b"5:hello";
        let res = decode_string(&mut reader).unwrap();
        assert_eq!("hello".to_string(), res);
    }

    #[test]
    fn test_decode_int() {
        let exp = Num::I64(-8848);
        let mut reader: &[u8] = b"i-8848e";
        let num = decode_int(&mut reader).unwrap();
        assert_eq!(exp, num);

        let exp = Num::U64(2314);
        let mut reader: &[u8] = b"i2314e";
        let num = decode_int(&mut reader).unwrap();
        assert_eq!(exp, num);
    }

    #[test]
    fn test_bytes_parse() {
        let expect_bytes = b"hello1234".to_vec();
        let mut bencode: &[u8] = b"l5:helloi1234ee";
        let res_bytes = parse_bencode_bytes(&mut bencode).unwrap();
        assert_eq!(expect_bytes, res_bytes);

        let expect_bytes = b"ab".to_vec();
        let mut bencode: &[u8] = b"d1:a1:be";
        let res_bytes = parse_bencode_bytes(&mut bencode).unwrap();
        assert_eq!(expect_bytes, res_bytes);
    }

    #[test]
    fn test_bencode_parse() {
        let mut expect_bencode: &[u8] = b"l4:Rusti1314el4:Java6:Golanged3:onei-1e3:twoi2eee";
        let expect_obj = BObject::BList(VecDeque::from([
            BObject::BStr(String::from("Rust")),
            BObject::BInt(Num::U64(1314)),
            BObject::BList(VecDeque::from([
                BObject::BStr(String::from("Java")),
                BObject::BStr(String::from("Golang")),
            ])),
            BObject::BDict(Map::from([
                (String::from("one"), BObject::BInt(Num::I64(-1))),
                (String::from("two"), BObject::BInt(Num::U64(2))),
            ])),
        ]));

        let res_obj = BObject::parse(&mut expect_bencode).unwrap();
        assert_eq!(expect_obj, res_obj);
    }
}
