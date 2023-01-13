use std::cmp::Ordering;

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
    #[default]
    None,
    ExactVersion(ExactVersion),
    Range,
}

impl Version {
    pub const fn is_none(&self) -> bool {
        matches!(self, Version::None)
    }
}

#[derive(Debug, Default, Eq, Ord)]
pub struct ExactVersion {
    pub(crate) operatator: Operator,
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
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
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

impl From<(u16, u16, u16)> for ExactVersion {
    fn from((major, minor, patch): (u16, u16, u16)) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            operatator: Operator::default(),
        }
    }
}

impl From<(u16, u16)> for ExactVersion {
    fn from((major, minor): (u16, u16)) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: None,
            operatator: Operator::default(),
        }
    }
}

impl From<u16> for ExactVersion {
    fn from(major: u16) -> Self {
        Self {
            major,
            minor: None,
            patch: None,
            operatator: Operator::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::version::ExactVersion;

    #[test]
    fn ordering() {
        assert_eq!(ExactVersion::from((0, 0, 0)), ExactVersion::from((0, 0, 0)));
        assert!(ExactVersion::from((0, 0, 1)) > ExactVersion::from((0, 0, 0)));
        assert!(ExactVersion::from((0, 0, 0)) < ExactVersion::from((0, 0, 1)));
        assert!(ExactVersion::from((0, 1, 0)) > ExactVersion::from((0, 0, 1)));
        assert_eq!(ExactVersion::from((0, 1, 0)), ExactVersion::from((0, 1, 0)));
        assert!(ExactVersion::from((0, 1, 0)) < ExactVersion::from((0, 2, 1)));
        assert!(ExactVersion::from((0, 2, 1)) > ExactVersion::from((0, 2, 0)));
        assert!(ExactVersion::from((1, 2, 1)) < ExactVersion::from((2, 2, 1)));
        assert!(ExactVersion::from((4, 2, 1)) > ExactVersion::from((2, 2, 1)));
    }
}
