use serde_json::Value;
use crate::error::Error;

pub trait DBCache {
    fn query(&mut self, json: &Value) -> Result<Vec<i32>, Error>;
    fn native_query(&mut self, query: &str) -> Result<Vec<i32>, Error>;
    fn get_download_links(&mut self, ids: Vec<i32>) -> Result<Vec<String>, Error>;
}
