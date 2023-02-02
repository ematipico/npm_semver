use crate::version::Operator;
use crate::{ExactVersion, SemverError, Version};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, digit1};
use nom::combinator::{map, map_res, opt};
use nom::error::{FromExternalError, ParseError};
use nom::Err;
use nom::IResult;
use std::num::ParseIntError;

pub fn parse(source: &str) -> Result<Version, SemverError> {
    let result = exact_version::<SemverError>(source);
    match result {
        Ok((_, exact_version)) => Ok(Version::ExactVersion(exact_version)),
        Err(err) => match err {
            Err::Error(e) | Err::Failure(e) => return Err(e),
            _ => unreachable!("It should be incomplete"),
        },
    }
}

fn exact_version<'a, E>(source: &'a str) -> IResult<&str, ExactVersion, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let (source, operator) = operator(source)?;
    let (source, major) = major(source)?;
    let (source, minor) = minor_or_patch(source)?;
    let (source, patch) = minor_or_patch(source)?;

    Ok((
        source,
        ExactVersion {
            operator: operator.unwrap_or_default(),
            major,
            minor,
            patch,
        },
    ))
}

fn operator<'a, E: ParseError<&'a str>>(source: &'a str) -> IResult<&'a str, Option<Operator>, E> {
    opt(alt((
        // first multi chars
        map(tag("<="), |_| Operator::LessEq),
        map(tag(">="), |_| Operator::GreaterEq),
        map(char('='), |_| Operator::Exact),
        map(char('<'), |_| Operator::Less),
        map(char('>'), |_| Operator::Greater),
        map(char('^'), |_| Operator::Caret),
        map(char('*'), |_| Operator::Wildcard),
        map(char('~'), |_| Operator::Tilde),
    )))(source)
}

fn major<'a, E: ParseError<&'a str>>(source: &'a str) -> IResult<&'a str, u16, E> {
    map(take_while1(|c| (c as char).is_numeric()), |result: &str| {
        result.parse::<u16>().unwrap()
    })(source)
}

fn minor_or_patch<'a, E>(source: &'a str) -> IResult<&'a str, Option<u16>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let (source, result) = opt(tag("."))(source)?;
    if result.is_some() {
        let (source, result) = map_res(digit1, |result: &str| result.parse::<u16>())(source)?;
        Ok((source, Some(result)))
    } else {
        Ok((source, None))
    }
}

#[cfg(test)]
mod test {
    use crate::parser::parse;
    use crate::{exact_version, ExactVersion, SemverError, Version};

    fn assert_ok_parse(source: &str, expected_version: Version) {
        let result = parse(source);
        assert!(result.is_ok(), "instead got {result:?}");
        match result {
            Ok(version) => {
                assert_eq!(version, expected_version);
            }
            Err(err) => {
                panic!("This should not error, instead got {err:?}");
            }
        }
    }

    fn assert_err_parse(source: &str, expected_error: SemverError) {
        let result = parse(source);
        assert!(result.is_err(), "instead got {result:?}");
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
    fn ok() {
        assert_ok_parse("1.0.0", Version::ExactVersion(exact_version!(1, 0, 0)));
        assert_ok_parse("1.0", Version::ExactVersion(exact_version!(1, 0)));
        assert_ok_parse("1", Version::ExactVersion(exact_version!(1)));
    }

    #[test]
    fn major_err() {
        assert_err_parse(
            "something",
            SemverError::NotANumber("something".to_string()),
        )
    }
    #[test]
    fn minor_err() {
        assert_err_parse(
            "1.something",
            SemverError::NotANumber("something".to_string()),
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
            SemverError::NotANumber("something".to_string()),
        )
    }
}
