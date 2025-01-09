use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid glob pattern: {pattern}")]
    InvalidGlobPattern {
        pattern: String,
        #[source]
        source: glob::PatternError,
    },

    #[error("Invalid regex pattern: {pattern}")]
    InvalidRegexPattern {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
