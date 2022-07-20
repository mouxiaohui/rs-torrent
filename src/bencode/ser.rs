use std::{collections::BTreeMap, io::Write};

use anyhow::Result;
use serde::{ser, Serialize};

use super::{
    bencode::{encode_bytes, encode_int},
    err::Error,
    ser_string,
};

pub fn to_bytes<T: ser::Serialize>(b: &T) -> Result<Vec<u8>> {
    let mut writer = Vec::new();
    b.serialize(&mut Serializer::new(&mut writer))?;
    Ok(writer)
}

#[derive(Debug)]
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }

    #[inline]
    pub fn into_vec(self) -> W {
        self.writer
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = SerializeMap<'a, W>;
    type SerializeStruct = SerializeMap<'a, W>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    #[inline]
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        encode_int(&mut self.writer, v)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        encode_int(&mut self.writer, v)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        encode_bytes(&mut self.writer, v.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_bytes(&mut self.writer, v)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.write_all(b"l")?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.write_all(b"d")?;
        Ok(SerializeMap::new(self))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}

pub struct SerializeMap<'a, W> {
    ser: &'a mut Serializer<W>,
    cur_key: Option<Vec<u8>>,
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<'a, W> SerializeMap<'a, W>
where
    W: Write,
{
    fn new(ser: &'a mut Serializer<W>) -> Self {
        Self {
            ser,
            cur_key: None,
            entries: BTreeMap::new(),
        }
    }

    #[inline]
    fn end_map(&mut self) -> Result<(), Error> {
        if self.cur_key.is_some() {
            return Err(Error::KeyWithoutValue);
        }

        for (k, v) in &self.entries {
            ser::Serializer::serialize_bytes(&mut *self.ser, k.as_ref())?;
            self.ser.writer.write_all(&v)?;
        }

        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for SerializeMap<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        if self.cur_key.is_some() {
            return Err(Error::KeyWithoutValue);
        }
        self.cur_key = Some(key.serialize(&mut ser_string::StringSerializer)?);
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let key = self.cur_key.take().ok_or(Error::ValueWithoutKey)?;
        let mut ser = Serializer::new(Vec::new());
        value.serialize(&mut ser)?;
        self.entries.insert(key, ser.into_vec());

        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_map()?;
        self.ser.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for SerializeMap<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let k = key.serialize(&mut ser_string::StringSerializer)?;
        let mut ser = Serializer::new(Vec::new());
        value.serialize(&mut ser)?;
        self.entries.insert(k, ser.into_vec());
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_map()?;
        self.ser.writer.write_all(b"e")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_derive::{Deserialize, Serialize};

    use super::to_bytes;

    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct User<'a> {
        name: &'a str,
        age: u64,
        class: Vec<&'a str>,
    }

    #[test]
    fn test_ser() {
        let user = User {
            name: "xiaohui",
            age: 18,
            class: vec!["RMIT", "Music"],
        };

        let bytes = to_bytes(&user).unwrap();
        let exp: &[u8] = b"d3:agei18e5:classl4:RMIT5:Musice4:name7:xiaohuie";
        assert_eq!(exp, bytes);
    }
}
