use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

use anyhow::{anyhow, Result};

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
    Ok(reader.fill_buf()?[0])
}

#[derive(Debug, PartialEq)]
pub enum Num {
    U64(u64),
    I64(i64),
}

impl Num {
    pub fn str_num(&self) -> String {
        match self {
            Num::U64(v) => v.to_string(),
            Num::I64(v) => v.to_string(),
        }
    }
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

    pub fn bencode<W: ?Sized>(&self, writer: &mut W) -> Result<usize>
    where
        W: Write,
    {
        match self {
            BObject::BStr(s) => encode_string(writer, s),
            BObject::BInt(i) => encode_int(writer, i),
            BObject::BList(list) => {
                let mut len = 2;
                writer.write(b"l")?;
                for obj in list {
                    len += obj.bencode(writer)?;
                }
                writer.write(b"e")?;
                writer.flush()?;

                Ok(len)
            }
            BObject::BDict(dict) => {
                let mut len = 2;
                writer.write(b"d")?;
                for (k, v) in dict.k.iter().zip(dict.v.iter()) {
                    len += encode_string(writer, k)?;
                    len += v.bencode(writer)?;
                }
                writer.write(b"e")?;
                writer.flush()?;

                Ok(len)
            }
        }
    }
}

fn encode_string<W: ?Sized>(writer: &mut W, val: &str) -> Result<usize>
where
    W: Write,
{
    let val_fmt = format!("{}:{}", val.len(), val);
    let size = writer.write(val_fmt.as_bytes())?;
    writer.flush()?;

    Ok(size)
}

fn decode_string<R: ?Sized>(reader: &mut R) -> Result<String>
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

    let mut buf = vec![0; len];
    reader.read(&mut buf)?;
    let res = std::str::from_utf8(&buf)?;

    Ok(res.to_string())
}

fn encode_int<W: ?Sized>(writer: &mut W, val: &Num) -> Result<usize>
where
    W: Write,
{
    let val_fmt = format!("i{}e", val.str_num());
    let size = writer.write(val_fmt.as_bytes())?;
    writer.flush()?;

    Ok(size)
}

fn decode_int<R: ?Sized>(reader: &mut R) -> Result<Num>
where
    R: Read,
{
    let mut buf = [0; 1];

    reader.read(&mut buf)?;
    if buf[0] != b'i' {
        return Err(anyhow!(Error::InvalidInteger));
    }

    let mut num = Vec::new();
    let mut is_negative = false;

    loop {
        reader.read(&mut buf)?;
        if buf[0] == b'e' {
            break;
        } else if buf[0] == b'-' {
            is_negative = true;
        }

        num.push(buf[0]);
    }

    if is_negative {
        return Ok(Num::I64(parse_utf8(&mut num)?));
    }

    Ok(Num::U64(parse_utf8(&mut num)?))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_string() {
        let mut writer = vec![];
        encode_string(&mut writer, "hello").unwrap();
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
        let num1 = Num::U64(8848);
        let num2 = Num::I64(-8848);
        encode_int(&mut writer, &num1).unwrap();
        assert_eq!("i8848e", std::str::from_utf8(&writer).unwrap());
        writer.clear();
        encode_int(&mut writer, &num2).unwrap();
        assert_eq!("i-8848e", std::str::from_utf8(&writer).unwrap());
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
    fn test_bencode_parse() {
        let mut buf = vec![];
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

        expect_obj.bencode(&mut buf).unwrap();
        assert_eq!(expect_bencode, &buf);

        let res_obj = BObject::parse(&mut expect_bencode).unwrap();
        assert_eq!(expect_obj, res_obj);
    }
}
