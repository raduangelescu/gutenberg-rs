use thiserror::Error;


#[derive(Error, Debug)]
pub enum ParseError {
    #[error("INVALID RESULT")]
    InvalidResult(String),
}