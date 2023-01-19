mod error;
pub(crate) mod macros;
mod parser;
mod version;

pub use error::ParseError;
pub use parser::parse;
pub use version::{ExactVersion, Version};
