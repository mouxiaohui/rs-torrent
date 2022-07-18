use std::{
    collections::VecDeque,
    io::{BufReader, Read},
};

use anyhow::Result;
use serde::{de, forward_to_deserialize_any};

use super::{
    bencode::{parse_bencode_bytes, BObject, Map, Num},
    err::Error,
};

pub fn from_bytes<'de, T>(b: &'de [u8]) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    de::Deserialize::deserialize(&mut Deserializer::new(b))
}

#[derive(Debug)]
pub struct Deserializer<R> {
    read: R,
    next: Option<BObject>,
}

impl<R> Deserializer<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        Self { read, next: None }
    }

    fn parse(&mut self) -> Result<BObject, Error> {
        if let Some(v) = self.next.take() {
            return Ok(v);
        }
        Ok(BObject::parse(&mut self.read).map_err(|e| Error::Custom(e.to_string()))?)
    }

    fn parse_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let mut reader = BufReader::new(&mut self.read);
        let bytes = parse_bencode_bytes(&mut reader).map_err(|e| Error::Custom(e.to_string()))?;
        Ok(bytes)
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    forward_to_deserialize_any! {
        string bool i8 i16 i32 u8 u16 u32 f32 f64 u64 i64 enum unit unit_struct
        tuple_struct ignored_any struct seq map
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.parse()? {
            BObject::BStr(v) => visitor.visit_string(v),
            BObject::BInt(v) => match v {
                Num::U64(v) => visitor.visit_u64(v),
                Num::I64(v) => visitor.visit_i64(v),
            },
            BObject::BList(v) => visitor.visit_seq(SeqAccess::new(self, v)),
            BObject::BDict(v) => visitor.visit_map(MapAccess::new(self, v)),
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.parse()? {
            BObject::BStr(v) => visitor.visit_str(v.as_str()),
            _ => Err(Error::InvalidStr)
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self.parse()? {
            BObject::BList(v) => {
                if v.len() != len {
                    Err(Error::InvalidList)
                } else {
                    Ok(v)
                }
            }
            _ => Err(Error::InvalidList),
        }?;

        visitor.visit_seq(SeqAccess::new(self, v))
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let bytes = self.parse_bytes()?;
        visitor.visit_byte_buf(bytes)
    }
}

#[derive(Debug)]
struct SeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    seq: VecDeque<BObject>,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, seq: VecDeque<BObject>) -> Self {
        Self { de, seq }
    }
}

impl<'de, 'a, R: Read + 'a> de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.seq.pop_front() {
            Some(v) => {
                self.de.next = Some(v);
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    map: Map<String, BObject>,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, map: Map<String, BObject>) -> Self {
        MapAccess { de, map }
    }
}

impl<'de, 'a, R: Read + 'a> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.map.k.pop_front() {
            Some(v) => {
                self.de.next = Some(BObject::BStr(v));
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.map.v.pop_front() {
            Some(v) => {
                self.de.next = Some(v);
                Ok(seed.deserialize(&mut *self.de)?)
            }
            None => Err(Error::InvalidDict),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    use super::*;

    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct User {
        name: String,
        age: u64,
    }

    #[test]
    fn test_parse_string() {
        let data: &[u8] = b"5:hello";
        let res: String = from_bytes(data).unwrap();
        assert_eq!("hello", res);
    }

    #[test]
    fn test_parse_int() {
        let data: &[u8] = b"i-23e";
        let res: i32 = from_bytes(data).unwrap();
        assert_eq!(-23, res);
    }

    #[test]
    fn test_parse_list() {
        let data: &[u8] = b"l1:A1:Be";
        let res: Vec<String> = from_bytes(data).unwrap();
        assert_eq!(Vec::from(["A", "B"]), res);
    }

    #[test]
    fn test_parse_dict() {
        let data: &[u8] = b"d3:cow3:moo4:spam4:eggse";
        let res: HashMap<String, String> = from_bytes(data).unwrap();
        let exp = HashMap::from([
            (String::from("cow"), String::from("moo")),
            (String::from("spam"), String::from("eggs")),
        ]);
        assert_eq!(exp, res);
    }

    #[test]
    fn test_parse_struct() {
        let user = User {
            name: "xiaohui".to_string(),
            age: 18,
        };
        let data: &[u8] = b"d4:name7:xiaohui3:agei18ee";
        let res: User = from_bytes(data).unwrap();
        assert_eq!(user, res);
    }
}
