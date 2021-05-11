use zip;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Unknown,
    Generic(String),
    Checksum(String, String),
    IO(std::io::Error),
    Http(reqwest::Error),
    UnknownGenre(String),
    ParseError(String),
    JSON(serde_json::Error),
    YoutubeDL(String),
    FFMPEG(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        use Error::*;

        match &self {
            &Unknown => write!(fmt, "unknown"),
            &Generic(ref s) => s.fmt(fmt),
            &Checksum(ref a, ref b) => write!(fmt, "mismatched checksum: '{}' != '{}'", a, b),
            &IO(ref err) => err.fmt(fmt),
            &Http(ref err) => err.fmt(fmt),
            &UnknownGenre(ref text) => write!(fmt, "unknown genre: '{}'", text),
            &ParseError(ref s) => write!(fmt, "failed to parse: '{}'", s),
            &JSON(ref err) => err.fmt(fmt),
            &YoutubeDL(ref err) => err.fmt(fmt),
            &FFMPEG(ref err) => err.fmt(fmt),
        }
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::Unknown
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_owned())
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

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JSON(err)
    }
}