use indicatif::style::TemplateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Rdf parsing error")]
    InvalidRdf(String),
    #[error("Invalid result")]
    InvalidResult(String),
    #[error("Settings error")]
    InvalidSettingsField(String),
    #[error("Io Error")]
    InvalidIO(String),
    #[error("Request error")]
    InvalidRequest(String),
    #[error("Progress bar template error")]
    InvalidProgressBarTemplate(String),
    #[error("SQLITE error")]
    InvalidSQLITE(String),
    #[error("UTF8 error")]
    InvalidUTF8String(String),
    #[error("Invalid Cache")]
    InvalidCacheLocation(String),
    #[error("Invalid URl")]
    InvalidUrl(String),
    #[error("Invalid Query")]
    InvalidQuery(String)
}

impl std::convert::From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err.to_string())
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::InvalidRequest(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::InvalidIO(err.to_string())
    }
}

impl std::convert::From<TemplateError> for Error {
    fn from(err: TemplateError) -> Self {
        Error::InvalidProgressBarTemplate(err.to_string())
    }
}

impl std::convert::From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::InvalidSQLITE(err.to_string())
    }
}

impl std::convert::From<fast_xml::Error> for Error {
    fn from(err: fast_xml::Error) -> Self {
        Error::InvalidRdf(err.to_string())
    }
}

impl std::convert::From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::InvalidUTF8String(err.to_string())
    }
}

impl std::convert::From<fast_xml::events::attributes::AttrError> for Error {
    fn from(err: fast_xml::events::attributes::AttrError) -> Self {
        Error::InvalidRdf(err.to_string())
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::InvalidRdf(err.to_string())
    }
}
