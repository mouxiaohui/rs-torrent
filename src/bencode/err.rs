use serde::de;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    InvalidType(String),
    InvalidStr,
    InvalidInteger,
    InvalidDict,
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Custom(s) => f.write_str(s),
            Error::InvalidType(s) => writeln!(f, "invalid type: {}", s),
            Error::InvalidStr => f.write_str("invalid bencode string"),
            Error::InvalidInteger => f.write_str("invalid bencode integer"),
            Error::InvalidDict => f.write_str("invalid dictionary"),
        }
    }
}
