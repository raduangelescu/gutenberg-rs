use std::error::Error;
use indexmap::IndexMap;

pub trait DBCache {
    fn query(&mut self, kwargs: IndexMap::<&str, &str>) -> Result<(), Box<dyn Error>>;
    fn native_query(&mut self, query: &str) -> Result<Vec<String>, Box<dyn Error>>;
}
