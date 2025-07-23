use serde::{de, ser};

use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedArray,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Eof => f.write_str("Unexpected end of input"),
            Error::Syntax => f.write_str("Invalid syntax"),
            Error::ExpectedBoolean => f.write_str("Expected boolean"),
            Error::ExpectedInteger => f.write_str("Expected integer"),
            Error::ExpectedString => f.write_str("Expected string"),
            Error::ExpectedArray => f.write_str("Expected array"),
            Error::ExpectedArrayEnd => f.write_str("Expected end of array"),
            Error::ExpectedMap => f.write_str("Expected map"),
            Error::ExpectedMapEnd => f.write_str("Expected end of map"),
            Error::ExpectedEnum => f.write_str("Expected enum"),
            Error::TrailingCharacters => f.write_str("Unexpected trailing characters"),
        }
    }
}

impl std::error::Error for Error {}
