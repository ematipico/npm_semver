use std::cmp::Ordering;

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
    #[default]
    None,
    ExactVersion(ExactVersion),
    Range(RangeVersion),
}

#[derive(Debug, Default, Ord, Eq)]
pub struct RangeVersion {
    min: ExactVersion,
    max: ExactVersion,
}

impl Version {
    pub const fn is_none(&self) -> bool {
        matches!(self, Version::None)
    }
}

#[derive(Debug, Default, Eq, Ord)]
pub struct ExactVersion {
    pub(crate) operator: Operator,
    pub(crate) patch: Option<u16>,
    pub(crate) minor: Option<u16>,
    pub(crate) major: u16,
}

impl ExactVersion {
    pub(crate) fn set_digit(&mut self, number: u16) {
        if self.major == u16::default() {
            self.major = number;
        } else if self.minor.is_none() {
            self.minor = Some(number)
        } else if self.patch.is_none() {
            self.patch = Some(number)
        }
    }

    pub(crate) fn set_operator(&mut self, operator: Operator) {
        self.operator = operator;
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
#[allow(dead_code)]
pub(crate) enum Operator {
    #[default]
    Exact,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Tilde,
    Caret,
    Wildcard,
}

impl PartialEq for ExactVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }

    fn ne(&self, other: &Self) -> bool {
        self.major != other.major || self.minor != other.minor || self.patch != other.patch
    }
}

impl PartialOrd for ExactVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            return Some(Ordering::Equal);
        }

        let result = if self.major > other.major {
            Ordering::Greater
        } else if self.major < other.major {
            Ordering::Less
        } else {
            if self.minor > other.minor {
                Ordering::Greater
            } else if self.minor < other.minor {
                Ordering::Less
            } else {
                if self.patch > other.patch {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
        };

        Some(result)
    }
}

impl PartialEq for RangeVersion {
    fn eq(&self, other: &Self) -> bool {
        self.min.eq(&other.min) && self.max.eq(&other.max)
    }
}

impl PartialOrd for RangeVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.min.eq(&other.min) && self.max.eq(&other.max) {
            return Some(Ordering::Equal);
        }

        if self.min.eq(&other.min) {
            return self.max.partial_cmp(&other.max);
        } else if self.max.eq(&other.max) {
            return self.min.partial_cmp(&other.min);
        } else {
            let result = self.min.partial_cmp(&other.min);
            if matches!(result, Some(Ordering::Equal)) {
                self.max.partial_cmp(&other.max)
            } else {
                result
            }
        }
    }
}

impl From<(&str, u16, u16, u16)> for ExactVersion {
    fn from((operator, major, minor, patch): (&str, u16, u16, u16)) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            operator: Operator::from(operator),
        }
    }
}

impl From<(u16, u16, u16)> for ExactVersion {
    fn from((major, minor, patch): (u16, u16, u16)) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            operator: Operator::default(),
        }
    }
}

impl From<(u16, u16)> for ExactVersion {
    fn from((major, minor): (u16, u16)) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: None,
            operator: Operator::default(),
        }
    }
}

impl From<u16> for ExactVersion {
    fn from(major: u16) -> Self {
        Self {
            major,
            minor: None,
            patch: None,
            operator: Operator::default(),
        }
    }
}

impl From<&str> for Operator {
    fn from(value: &str) -> Self {
        match value {
            ">" => Operator::Greater,
            ">=" => Operator::GreaterEq,
            "<" => Operator::Less,
            "<=" => Operator::LessEq,
            "~" => Operator::Tilde,
            "=" => Operator::Exact,
            "*" => Operator::Wildcard,
            "^" => Operator::Caret,
            _ => Operator::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::version::{ExactVersion, RangeVersion};
    use crate::{exact_version, range_ver};

    #[test]
    fn ordering_range_version() {
        assert_eq!(
            range_ver!(exact_version!(0, 0, 0), exact_version!(0, 0, 0)),
            range_ver!(exact_version!(0, 0, 0), exact_version!(0, 0, 0))
        );

        assert!(
            range_ver!(exact_version!(0, 0, 1), exact_version!(0, 0, 0))
                > range_ver!(exact_version!(0, 0, 0), exact_version!(0, 0, 0))
        );

        assert!(
            range_ver!(exact_version!(0, 0, 1), exact_version!(0, 0, 0))
                < range_ver!(exact_version!(0, 0, 2), exact_version!(0, 0, 0))
        );
    }

    #[test]
    fn ordering_exact_version() {
        assert_eq!(exact_version!(0, 0, 0), exact_version!(0, 0, 0));
        assert!(exact_version!(0, 0, 1) > exact_version!(0, 0, 0));
        assert!(exact_version!(0, 0, 0) < exact_version!(0, 0, 1));
        assert!(exact_version!(0, 1, 0) > exact_version!(0, 0, 1));
        assert_eq!(exact_version!(0, 1, 0), exact_version!(0, 1, 0));
        assert!(exact_version!(0, 1, 0) < exact_version!(0, 2, 1));
        assert!(exact_version!(0, 2, 1) > exact_version!(0, 2, 0));
        assert!(exact_version!(1, 2, 1) < exact_version!(2, 2, 1));
        assert!(exact_version!(4, 2, 1) > exact_version!(2, 2, 1));
    }
}
