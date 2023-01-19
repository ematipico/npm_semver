use crate::error::ParseError;
use crate::version::{ExactVersion, Version};
use std::iter::{FusedIterator, Peekable};
use std::str::Chars;

struct Parser<'a> {
    #[allow(dead_code)]
    source: &'a str,
    versions: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.versions.next()
    }
}

impl<'a> FusedIterator for Parser<'a> {}

impl<'a> Parser<'a> {
    fn peek(&mut self) -> Option<&char> {
        self.versions.peek()
    }

    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            versions: source.chars().peekable(),
        }
    }

    fn parse(&mut self) -> Result<Version, ParseError> {
        let version = Version::default();
        match self.peek() {
            Some(_) => self.inner_parse(),
            None => {
                if version.is_none() {
                    Err(ParseError::Empty)
                } else {
                    Ok(version)
                }
            }
        }
    }

    fn inner_parse(&mut self) -> Result<Version, ParseError> {
        let mut exact_version = ExactVersion::default();
        let mut digit_chars = vec![];
        while let Some(piece) = self.next() {
            if piece != '.' {
                digit_chars.push(piece);
            } else {
                let digit = digit_chars.drain(0..).collect::<String>();
                let result = digit
                    .parse::<u16>()
                    .or_else(|_| Err(ParseError::NotANumber(digit)))?;
                exact_version.set_digit(result);
            }
        }
        if !digit_chars.is_empty() {
            let digit = digit_chars.drain(0..).collect::<String>();
            let result = digit
                .parse::<u16>()
                .or_else(|_| Err(ParseError::NotANumber(digit)))?;
            exact_version.set_digit(result);
        }

        Ok(Version::ExactVersion(exact_version))
    }
}

pub fn parse(input: &str) -> Result<Version, ParseError> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use crate::error::ParseError;
    use crate::parser::parse;
    use crate::version::Version;

    fn assert_ok_parse(source: &str, expected_version: Version) {
        let result = parse(source);
        assert!(result.is_ok());
        match result {
            Ok(version) => {
                assert_eq!(version, expected_version);
            }
            Err(err) => {
                panic!("This should not error, instead got {err:?}");
            }
        }
    }

    fn assert_err_parse(source: &str, expected_error: ParseError) {
        let result = parse(source);
        assert!(result.is_err());
        match result {
            Ok(version) => {
                panic!("This error, instead got {version:?}");
            }
            Err(err) => {
                assert_eq!(err, expected_error);
            }
        }
    }

    #[test]
    fn major_ok() {
        assert_ok_parse("1", Version::ExactVersion(1.into()))
    }

    #[test]
    fn major_and_minor_ok() {
        assert_ok_parse("1.1", Version::ExactVersion((1, 1).into()))
    }

    #[test]
    fn major_and_minor_patch_ok() {
        assert_ok_parse("1.56.3", Version::ExactVersion((1, 56, 3).into()))
    }

    #[test]
    fn major_err() {
        assert_err_parse("something", ParseError::NotANumber("something".to_string()))
    }
    #[test]
    fn minor_err() {
        assert_err_parse(
            "1.something",
            ParseError::NotANumber("something".to_string()),
        )
    }

    #[test]
    fn patch_err() {
        assert_err_parse(
            "1.1.something",
            ParseError::NotANumber("something".to_string()),
        )
    }
}
