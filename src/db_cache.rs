use std::error::Error;
use serde_json::Value;

pub trait DBCache {
    fn query(&mut self, json: &Value) -> Result<Vec<i32>, Box<dyn Error>>;
    fn native_query(&mut self, query: &str) -> Result<Vec<i32>, Box<dyn Error>>;
    fn get_download_links(&mut self, ids: Vec<i32>) -> Result<Vec<String>, Box<dyn Error>>;
}
