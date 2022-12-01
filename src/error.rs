use indicatif::style::TemplateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Rdf parsing error: `{0}`")]
    InvalidRdf(String),
    #[error("Invalid result: `{0}`")]
    InvalidResult(String),
    #[error("Settings error: `{0}`")]
    InvalidSettingsField(String),
    #[error("Io Error: `{0}`")]
    InvalidIO(String),
    #[error("Request error: `{0}`")]
    InvalidRequest(String),
    #[error("Progress bar template error: `{0}`")]
    InvalidProgressBarTemplate(String),
    #[error("SQLITE error: `{0}`")]
    InvalidSQLITE(String),
    #[error("UTF8 error: `{0}`")]
    InvalidUTF8String(String),
    #[error("Invalid Cache: `{0}`")]
    InvalidCacheLocation(String),
    #[error("Invalid URl: `{0}`")]
    InvalidUrl(String),
    #[error("Invalid Query: `{0}`")]
    InvalidQuery(String),
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

impl std::convert::From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Error::InvalidRdf(format!("fastxml error: {}", err.to_string()).to_string())
    }
}

impl std::convert::From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::InvalidUTF8String(err.to_string())
    }
}

impl std::convert::From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Error::InvalidRdf(format!("fastxml attribute error: {}", err.to_string()).to_string())
    }
}

/*impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::InvalidRdf(err.to_string())
    }
}*/
