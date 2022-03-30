use std::convert::From;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("given path to `rust` does not exist: {0}")]
    NoRust(std::path::PathBuf),
    #[error("given path to `gccrs` does not exist: {0}")]
    NoGccrs(std::path::PathBuf),
    #[error("{0}")]
    PathPrefix(std::path::StripPrefixError),
    #[error("{0}")]
    WalkDir(walkdir::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(e: std::path::StripPrefixError) -> Self {
        Error::PathPrefix(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Self {
        Error::WalkDir(e)
    }
}
