pub trait DBCache {
    fn create_cache(&mut self, parse_results: &Vec<String>);
    fn query(&self);
    fn native_query(self, query: &str);
}