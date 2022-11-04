use thiserror::Error;


#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid first item to double")]
    InvalidResult(String),
}