use zip;

#[derive(Debug)]
pub enum Error {
    Checksum(String, String),
    IO(std::io::Error),
    Http(reqwest::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match &self {
            &Error::Checksum(ref a, ref b) => write!(fmt, "mismatched checksum: {} != {}", a, b),
            &Error::IO(ref err) => err.fmt(fmt),
            &Error::Http(ref err) => err.fmt(fmt),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Error::IO(err.into())
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::IO(err.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;