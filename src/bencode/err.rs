#[derive(Debug)]
pub enum BencodeError {
    TypeError,
    InvalidBencode,
}

impl std::error::Error for BencodeError {}

impl std::fmt::Display for BencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BencodeError::TypeError => writeln!(f, "Wrong Type"),
            BencodeError::InvalidBencode => writeln!(f, "Invalid Bencode"),
        }
    }
}
