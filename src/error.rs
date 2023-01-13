use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Eq, PartialEq)]
pub enum ParseError {
    Empty,
    NotANumber(String),
}

impl Error for ParseError {}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => {
                write!(f, "Empty string")
            }
            ParseError::NotANumber(source) => {
                write!(f, "{source} is not a number")
            }
        }
    }
}
