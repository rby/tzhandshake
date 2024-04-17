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
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(std::fmt::format(format_args!(
            "BinSerialization failed dues to {}",
            msg
        )))
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
