use std::fmt;

#[derive(Debug)]
pub enum ConvertError {
    Io(std::io::Error),
    Decode(String),
    Encode(String),
    UnsupportedFormat(String),
    UnsupportedCodec(String),
    Resample(String),
    Tags(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::Io(e) => write!(f, "io error: {}", e),
            ConvertError::Decode(e) => write!(f, "decode error: {}", e),
            ConvertError::Encode(e) => write!(f, "encode error: {}", e),
            ConvertError::UnsupportedFormat(s) => write!(f, "unsupported format: {}", s),
            ConvertError::UnsupportedCodec(s) => write!(f, "unsupported codec: {}", s),
            ConvertError::Resample(e) => write!(f, "resample error: {}", e),
            ConvertError::Tags(e) => write!(f, "tags error: {}", e),
        }
    }
}

impl std::error::Error for ConvertError {}

impl From<std::io::Error> for ConvertError {
    fn from(e: std::io::Error) -> Self {
        ConvertError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, ConvertError>;
