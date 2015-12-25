extern crate byteorder;

use std::io;
use std::string;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pos: u64,
    kind: ErrorKind,
}

impl Error {
    pub fn new<K: Into<ErrorKind>>(pos: u64, kind: K) -> Self {
        Error {
            pos: pos,
            kind: kind.into(),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    FromUtf8Error(string::FromUtf8Error),
    InvalidMagic,
    Io(io::Error),
    DataError(String),
    UnexpectedEof,
    Unimplemented(String),
}

impl From<string::FromUtf8Error> for ErrorKind {
    fn from(err: string::FromUtf8Error) -> ErrorKind {
        ErrorKind::FromUtf8Error(err)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::Io(err)
    }
}

impl From<byteorder::Error> for ErrorKind {
    fn from(err: byteorder::Error) -> ErrorKind {
        match err {
            byteorder::Error::UnexpectedEOF => ErrorKind::UnexpectedEof,
            byteorder::Error::Io(err) => ErrorKind::Io(err),
        }
    }
}
