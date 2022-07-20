use std::{fmt::Display, io};

use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    Custom(String),
    Serialize(String),
    InvalidType(String),
    InvalidStr,
    InvalidInteger,
    InvalidList,
    InvalidDict,
    UnsupportedType,
    KeyWithoutValue,
    ValueWithoutKey,
    IoError(io::Error),
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self::Custom(e.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serialize(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Custom(s) => f.write_str(s),
            Error::Serialize(s) => f.write_str(s),
            Error::InvalidType(s) => writeln!(f, "invalid type: {}", s),
            Error::InvalidStr => f.write_str("invalid bencode string"),
            Error::InvalidInteger => f.write_str("invalid bencode integer"),
            Error::InvalidDict => f.write_str("invalid dictionary"),
            Error::InvalidList => f.write_str("invalid list"),
            Error::UnsupportedType => f.write_str("unsupported type"),
            Error::KeyWithoutValue => f.write_str("key without value"),
            Error::ValueWithoutKey => f.write_str("value without key"),
            Error::IoError(e) => f.write_str(&e.to_string()),
        }
    }
}
