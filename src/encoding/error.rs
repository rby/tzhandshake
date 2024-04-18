use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("String longer that u16")]
    StringTooLong,
    #[error("Custom({0})")]
    Custom(String),
    #[error("Maximum size reached: was {before}, added: {added}")]
    SizeOverflow { before: u16, added: u16 },
    #[error("IO error: `{0}`")]
    IO(#[from] std::io::Error),
    #[error("UTF decoding error: `{0}`")]
    FormUTF8(#[from] std::string::FromUtf8Error),
    #[error("Too many bytes")]
    ExtraBytes,
    #[error("Not enough bytes")]
    UnsufficentBytes,
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(std::fmt::format(format_args!(
            "BinSerialization failed due to {}",
            msg
        )))
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(std::fmt::format(format_args!(
            "BinSerialization failed due to {}",
            msg
        )))
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
