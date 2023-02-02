use crate::version::Operator;
use crate::{ExactVersion, SemverError, Version};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, digit1};
use nom::combinator::{map, map_res, opt};
use nom::error::{context, ContextError, FromExternalError, ParseError};
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::{Err, IResult};
use std::fmt::Debug;
use std::num::ParseIntError;

/// Parses a single semver version
///
/// ## Error
///
///
pub fn parse_version(source: &str) -> Result<Version, SemverError> {
    let result = exact_version::<SemverError>(source);
    match result {
        Ok((_, exact_version)) => Ok(Version::ExactVersion(exact_version)),
        Err(err) => match err {
            Err::Error(e) | Err::Failure(e) => Err(e),
            _ => unreachable!("It should be incomplete"),
        },
    }
}

/// Parses a series of [Version] separated by "||"
///
/// ## Error
pub fn parse_versions(source: &str) -> Result<Vec<Version>, SemverError> {
    let result = versions::<SemverError>(source);
    match result {
        Ok((_, versions)) => Ok(versions),
        Err(err) => match err {
            Err::Error(e) | Err::Failure(e) => Err(e),
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
    map(take_while1(|c: char| c.is_numeric()), |result: &str| {
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

/// Parses a space
fn space<'a, E: ParseError<&'a str>>(source: &'a str) -> IResult<&'a str, &'a str, E> {
    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| c == ' ')(source)
}

/// Parses the logical OR "||" inside a string
fn or<'a, E: ParseError<&'a str> + Debug>(source: &'a str) -> IResult<&'a str, &'a str, E> {
    tag("||")(source)
}

fn versions<'a, E>(source: &'a str) -> IResult<&'a str, Vec<Version>, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, ParseIntError>
        + ContextError<&'a str>
        + Debug,
{
    context(
        "versions",
        separated_list1(
            delimited(space, or, space),
            map(exact_version, Version::ExactVersion),
        ),
    )(source)
}

#[cfg(test)]
mod test {
    use crate::parser::{parse_version, parse_versions};
    use crate::{exact_version, ExactVersion, SemverError, Version};

    fn assert_ok_parse(source: &str, expected_version: Version) {
        let result = parse_version(source);
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
        let result = parse_version(source);
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

    #[test]
    fn versions_ok() {
        let result = parse_versions("1.0.0 ||2.0.0");
        assert!(result.is_ok(), "instead {result:?}");
        let versions = result.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0], Version::ExactVersion(exact_version!(1, 0, 0)));
        assert_eq!(versions[1], Version::ExactVersion(exact_version!(2, 0, 0)));
    }

    #[test]
    fn versions_err() {
        let result = parse_versions("1.0.0xx2");
        assert!(result.is_err(), "instead {result:?}");
    }
}
