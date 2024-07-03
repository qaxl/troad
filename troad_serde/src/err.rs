use std::fmt::Display;

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),

    Eof,
    UnexpectedEof(usize, usize),
    BadUtf8Input,
}

impl ser::Error for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error::Message(message.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error::Message(message.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::UnexpectedEof(a, b) => f.write_fmt(format_args!("unexpected end of input ({a}, {b})")),
            Error::BadUtf8Input => f.write_str("bad utf-8 input"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::other(value)
    }
}
