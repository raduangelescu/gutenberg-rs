use crate::fst_parser::ParseResult;

pub trait DBCache {
    fn create_cache(&mut self, parse_results: &ParseResult);
    fn query(&self);
    fn native_query(self, query: &str);
}