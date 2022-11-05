use crate::fst_parser::ParseResult;
use std::error::Error;
use indexmap::IndexMap;

pub trait DBCache {
    fn create_cache(&mut self, parse_results: &ParseResult) -> Result<(), Box<dyn Error>>;
    fn query(&self, kwargs: IndexMap::<&str, Vec<&str>>) -> Result<(), Box<dyn Error>>;
    fn native_query(self, query: &str) -> Result<(), Box<dyn Error>>;
}
