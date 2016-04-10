//! Error codes

use std::fmt;
use std::io;
use std::error;
use std::result;
use byteorder;
use serde::{ser, de};

#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode {
    /// Unsupported opcode
    Unsupported(char),
    /// EOF while parsing op argument
    EOFWhileParsing,
    /// Stack underflowed
    StackUnderflow,
    /// Length prefix found negative
    NegativeLength,
    /// String decoding as UTF-8 failed
    StringNotUTF8,
    /// Wrong stack top type for opcode
    InvalidStackTop,
    /// Value not hashable, but used as dict key or set item
    ValueNotHashable,
    /// Invalid literal found
    InvalidLiteral(Vec<u8>),
    /// Found trailing bytes after STOP opcode
    TrailingBytes,
    /// Invalid type encountered
    InvalidType(de::Type),
    /// Invalid value encountered
    InvalidValue(String),
    /// Invalid length encountered
    InvalidLength(usize),
    /// Unknown enum variant
    UnknownVariant(String),
    /// Unknown field
    UnknownField(String),
    /// Missing field
    MissingField(&'static str),
    /// Custom error
    Custom(String),
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Unsupported(ch) => write!(fmt, "unsupported opcode {:?}", ch),
            ErrorCode::EOFWhileParsing => write!(fmt, "EOF while parsing"),
            ErrorCode::StackUnderflow => write!(fmt, "pickle stack underflow"),
            ErrorCode::NegativeLength => write!(fmt, "negative length prefix"),
            ErrorCode::StringNotUTF8 => write!(fmt, "string is not UTF8 encoded"),
            ErrorCode::InvalidStackTop => write!(fmt, "invalid type of top of stack"),
            ErrorCode::ValueNotHashable => write!(fmt, "dict key or set item not hashable"),
            ErrorCode::InvalidLiteral(ref l) => write!(fmt, "literal is invalid: {}",
                                                       String::from_utf8_lossy(&l)),
            ErrorCode::TrailingBytes => write!(fmt, "trailing bytes found"),
            ErrorCode::InvalidType(ref t) => write!(fmt, "invalid type: {:?}", t),
            ErrorCode::InvalidValue(ref s) => write!(fmt, "invalid value: {}", s),
            ErrorCode::InvalidLength(l) => write!(fmt, "invalid length: {}", l),
            ErrorCode::UnknownVariant(ref v) => write!(fmt, "unknown variant: {}", v),
            ErrorCode::UnknownField(ref f) => write!(fmt, "unknown field: {}", f),
            ErrorCode::MissingField(f) => write!(fmt, "missing field: {}", f),
            ErrorCode::Custom(ref s) => fmt.write_str(s),
        }
    }
}

/// This type represents all possible errors that can occur when serializing or
/// deserializing a value.
#[derive(Debug)]
pub enum Error {
    /// Some IO error occurred when serializing or deserializing a value.
    Io(io::Error),
    /// The pickle had some error while interpreting.
    Eval(ErrorCode, usize),
    /// Syntax error while transforming into Rust values.
    Syntax(ErrorCode),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<byteorder::Error> for Error {
    fn from(error: byteorder::Error) -> Error {
        match error {
            byteorder::Error::Io(err) => Error::Io(err),
            byteorder::Error::UnexpectedEOF => Error::Io(io::Error::new(io::ErrorKind::UnexpectedEof, error)),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref error) => error.fmt(fmt),
            Error::Eval(ref code, offset) => write!(fmt, "eval error at offset {}: {}",
                                                    offset, code),
            Error::Syntax(ref code) => write!(fmt, "decoding error: {}", code)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref error) => error::Error::description(error),
            Error::Eval(..) => "pickle eval error",
            Error::Syntax(..) => "serde decoding error",
        }
    }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Syntax(ErrorCode::Custom(msg.into()))
    }

    fn end_of_stream() -> Error {
        Error::Syntax(ErrorCode::EOFWhileParsing)
    }

    fn invalid_type(ty: de::Type) -> Error {
        Error::Syntax(ErrorCode::InvalidType(ty))
    }

    fn invalid_value(msg: &str) -> Error {
        Error::Syntax(ErrorCode::InvalidValue(String::from(msg)))
    }

    fn invalid_length(len: usize) -> Error {
        Error::Syntax(ErrorCode::InvalidLength(len))
    }

    fn unknown_variant(variant: &str) -> Error {
        Error::Syntax(ErrorCode::UnknownVariant(String::from(variant)))
    }

    fn unknown_field(field: &str) -> Error {
        Error::Syntax(ErrorCode::UnknownField(String::from(field)))
    }

    fn missing_field(field: &'static str) -> Error {
        Error::Syntax(ErrorCode::MissingField(field))
    }
}

impl ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Syntax(ErrorCode::Custom(msg.into()))
    }
}