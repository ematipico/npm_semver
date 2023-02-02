mod error;
pub(crate) mod macros;
mod parser;
mod version;

use crate::parser::parse_versions;
pub use error::SemverError;
pub use parser::parse_version;
pub use version::{ExactVersion, Version};

pub fn satisfies(first_version: &str, ranges: &str) -> Result<bool, SemverError> {
    let first_version = parse_version(first_version)?;
    let ranges = parse_versions(ranges)?;

    Ok(ranges
        .iter()
        .any(|range_version| &first_version >= range_version))
}

#[cfg(test)]
mod test {
    use crate::satisfies;

    #[test]
    fn satisfies_ok() {
        assert!(satisfies("1.1.0", "1.0.0").unwrap());
        assert!(satisfies("1.1.0", "1.1.0").unwrap());
        assert!(satisfies("1.6.0", "1.0.0").unwrap());
        assert!(satisfies("2.0.0", "1.0.0").unwrap());
    }
}
