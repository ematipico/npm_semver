use crate::error::ParseError;
use crate::version::{ExactVersion, Operator, Version};
use std::iter::{Enumerate, FusedIterator, Peekable};
use std::str::Chars;

struct Parser<'a> {
    #[allow(dead_code)]
    source: &'a str,
    versions: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.versions.next()
    }
}

impl<'a> FusedIterator for Parser<'a> {}

impl<'a> Parser<'a> {
    fn peek(&mut self) -> Option<&(usize, char)> {
        self.versions.peek()
    }

    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            versions: source.chars().enumerate().peekable(),
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

        while let Some((index, piece)) = self.next() {
            if let Some(operator) = self.extract_operator(index, piece) {
                exact_version.set_operator(operator);
                if matches!(operator, Operator::GreaterEq | Operator::LessEq) {
                    self.next();
                }
            } else {
                match piece {
                    '.' => {
                        let digit = digit_chars.drain(0..).collect::<String>();
                        let result = digit
                            .parse::<u16>()
                            .or_else(|_| Err(ParseError::NotANumber(digit)))?;
                        exact_version.set_digit(result);
                    }

                    _ => {
                        digit_chars.push(piece);
                    }
                }
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

    fn extract_operator(&mut self, index: usize, piece: char) -> Option<Operator> {
        if index == 0 {
            let operator = match piece {
                '>' if self
                    .peek()
                    .map(|(_, char)| *char == '=')
                    .unwrap_or_default() =>
                {
                    Operator::GreaterEq
                }
                '<' if self
                    .peek()
                    .map(|(_, char)| *char == '=')
                    .unwrap_or_default() =>
                {
                    Operator::LessEq
                }
                '>' => Operator::Greater,
                '<' => Operator::Less,
                '~' => Operator::Tilde,
                '^' => Operator::Caret,
                '=' => Operator::Exact,
                _ => return None,
            };

            Some(operator)
        } else {
            None
        }
    }
}

pub fn parse(input: &str) -> Result<Version, ParseError> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use crate::error::ParseError;
    use crate::exact_version;
    use crate::parser::parse;
    use crate::version::{ExactVersion, Version};

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
    fn operators_ok() {
        assert_ok_parse(
            ">1.0.0",
            Version::ExactVersion(exact_version!(">", 1, 0, 0)),
        );
        assert_ok_parse(
            ">=1.0.0",
            Version::ExactVersion(exact_version!(">=", 1, 0, 0)),
        );
        assert_ok_parse(
            "<1.0.0",
            Version::ExactVersion(exact_version!("<", 1, 0, 0)),
        );
        assert_ok_parse(
            "<=1.0.0",
            Version::ExactVersion(exact_version!("<=", 1, 0, 0)),
        );
        assert_ok_parse(
            "~1.0.0",
            Version::ExactVersion(exact_version!("~", 1, 0, 0)),
        );
        assert_ok_parse(
            "^1.0.0",
            Version::ExactVersion(exact_version!("^", 1, 0, 0)),
        );
        assert_ok_parse(
            "=1.0.0",
            Version::ExactVersion(exact_version!("=", 1, 0, 0)),
        );
    }

    #[test]
    fn patch_err() {
        assert_err_parse(
            "1.1.something",
            ParseError::NotANumber("something".to_string()),
        )
    }
}
