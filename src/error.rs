use nom::error::{ErrorKind, FromExternalError, ParseError};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;

#[derive(Eq, PartialEq)]
pub enum SemverError {
    Empty,
    NotANumber(String),
}

impl Error for SemverError {}

impl Debug for SemverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for SemverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SemverError::Empty => {
                write!(f, "Empty string")
            }
            SemverError::NotANumber(source) => {
                write!(f, "{source} is not a number")
            }
        }
    }
}

impl ParseError<&str> for SemverError {
    fn from_error_kind(input: &str, _kind: ErrorKind) -> Self {
        SemverError::NotANumber(input.to_string())
    }

    fn append(input: &str, _kind: ErrorKind, _other: Self) -> Self {
        SemverError::NotANumber(input.to_string())
    }
}

impl FromExternalError<&str, ParseIntError> for SemverError {
    fn from_external_error(input: &str, _kind: ErrorKind, _e: ParseIntError) -> Self {
        Self::NotANumber(input.to_string())
    }
}
