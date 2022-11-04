use crate::fst_parser::ParseResult;
use std::error::Error;

pub trait DBCache {
    fn create_cache(&mut self, parse_results: &ParseResult) -> Result<(), Box<dyn Error>>;
    fn query(&self) -> Result<(), Box<dyn Error>>;
    fn native_query(self, query: &str) -> Result<(), Box<dyn Error>>;
}