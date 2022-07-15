use std::{
    collections::HashMap,
    io::{BufRead, Read, Write},
    str::FromStr,
};

use anyhow::{anyhow, Result};

use super::err::BencodeError;

fn parse_utf8<F>(v: &[u8]) -> Result<F>
where
    F: FromStr,
{
    match std::str::from_utf8(v)?.parse::<F>() {
        Ok(res) => Ok(res),
        Err(_) => Err(anyhow!("Parse failure: {:?}", v)),
    }
}

fn peek(reader: &mut dyn BufRead) -> Result<u8> {
    Ok(reader.fill_buf()?[0].clone())
}

#[derive(Debug, PartialEq)]
pub struct Num {
    sign: bool,
    val: u64,
}

impl Num {
    pub fn new(sign: bool, val: u64) -> Self {
        Self { sign, val }
    }

    pub fn sign(&self) -> String {
        match self.sign {
            true => String::from("-"),
            false => String::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BObject {
    BStr(String),
    BInt(Num),
    BList(Vec<Box<BObject>>),
    BDict(HashMap<String, Box<BObject>>),
}

impl BObject {
    pub fn parse<R: BufRead>(reader: &mut R) -> Result<Self> {
        match peek(reader)? {
            b'0'..=b'9' => Ok(Self::BStr(decode_string(reader)?)),
            b'i' => Ok(Self::BInt(decode_int(reader)?)),
            b'l' => {
                reader.consume(1);
                let mut list = Vec::new();
                loop {
                    if peek(reader)? == b'e' {
                        reader.consume(1);
                        break;
                    }
                    let obj = Self::parse(reader)?;
                    list.push(Box::new(obj));
                }

                Ok(Self::BList(list))
            }
            b'd' => {
                reader.consume(1);
                let mut dict = HashMap::new();
                loop {
                    if peek(reader)? == b'e' {
                        reader.consume(1);
                        break;
                    }
                    let key = decode_string(reader)?;
                    let val = Self::parse(reader)?;
                    dict.insert(key, Box::new(val));
                }

                Ok(Self::BDict(dict))
            }
            _ => Err(anyhow!(BencodeError::InvalidBencode)),
        }
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
                for (k, obj) in dict {
                    len += encode_string(writer, k)?;
                    len += obj.bencode(writer)?;
                }
                writer.write(b"e")?;
                writer.flush()?;

                Ok(len)
            }
        }
    }
}

pub fn encode_string<W: ?Sized>(writer: &mut W, val: &str) -> Result<usize>
where
    W: Write,
{
    let val_fmt = format!("{}:{}", val.len(), val);
    let size = writer.write(val_fmt.as_bytes())?;
    writer.flush()?;

    Ok(size)
}

pub fn decode_string<R: ?Sized>(reader: &mut R) -> Result<String>
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

        len = (len * 10) + parse_utf8::<usize>(&buf)?;
    }

    let mut buf = vec![0; len];
    reader.read(&mut buf)?;
    let res = std::str::from_utf8(&buf)?;

    Ok(res.to_string())
}

pub fn encode_int<W: ?Sized>(writer: &mut W, val: &Num) -> Result<usize>
where
    W: Write,
{
    let val_fmt = format!("i{}{}e", val.sign(), val.val);
    let size = writer.write(val_fmt.as_bytes())?;
    writer.flush()?;

    Ok(size)
}

pub fn decode_int<R: ?Sized>(reader: &mut R) -> Result<Num>
where
    R: Read,
{
    let mut buf = [0; 1];

    reader.read(&mut buf)?;
    if buf[0] != b'i' {
        return Err(BencodeError::TypeError.into());
    }

    let mut num = Vec::new();
    let mut sign = false;
    loop {
        reader.read(&mut buf)?;
        if buf[0] == b'e' {
            break;
        } else if buf[0] == b'-' {
            sign = true;
        } else {
            num.push(buf[0]);
        }
    }

    Ok(Num {
        sign,
        val: parse_utf8::<u64>(&num)?,
    })
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
        let num = Num::new(false, 8848);
        encode_int(&mut writer, &num).unwrap();
        assert_eq!("i8848e", std::str::from_utf8(&writer).unwrap());
    }

    #[test]
    fn test_decode_int() {
        let exp = Num::new(true, 8848);
        let mut reader: &[u8] = b"i-8848e";
        let num = decode_int(&mut reader).unwrap();
        assert_eq!(exp, num);
    }

    #[test]
    fn test_bencode_parse() {
        let mut buf = vec![];
        let expect_obj = BObject::BList(vec![
            Box::new(BObject::BStr(String::from("Rust"))),
            Box::new(BObject::BInt(Num::new(true, 12138))),
            Box::new(BObject::BList(vec![
                Box::new(BObject::BStr(String::from("Java"))),
                Box::new(BObject::BStr(String::from("Golang"))),
            ])),
            Box::new(BObject::BDict(HashMap::from([
                (
                    String::from("one"),
                    Box::new(BObject::BInt(Num::new(false, 1))),
                ),
                (
                    String::from("two"),
                    Box::new(BObject::BInt(Num::new(false, 2))),
                ),
            ]))),
        ]);

        expect_obj.bencode(&mut buf).unwrap();
        let res_obj = BObject::parse(&mut &buf[..]).unwrap();
        assert_eq!(expect_obj, res_obj);
    }
}
