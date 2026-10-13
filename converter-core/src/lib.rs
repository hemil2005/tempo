pub mod decode;
pub mod encode;
pub mod error;
pub mod format;
pub mod tags;

pub use decode::{decode_file, Decoded};
pub use encode::convert;
pub use error::{ConvertError, Result};
pub use format::{AudioFormat, EncodeSettings, OutputFormat, Quality};
pub use tags::{Picture, Tags};
