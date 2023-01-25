#[macro_export]
macro_rules! exact_version {
    ($major:literal) => {
        ExactVersion::from($major)
    };
    ($major:literal, $minor:literal) => {
        ExactVersion::from(($major, $minor))
    };
    ($major:literal, $minor:literal, $patch:literal) => {
        ExactVersion::from(($major, $minor, $patch))
    };
    ($operator:literal, $major:literal, $minor:literal, $patch:literal) => {
        ExactVersion::from(($operator, $major, $minor, $patch))
    };
}

#[macro_export]
macro_rules! range_ver {
    ($min:expr, $max:expr) => {
        RangeVersion {
            min: $min,
            max: $max,
        }
    };
}
