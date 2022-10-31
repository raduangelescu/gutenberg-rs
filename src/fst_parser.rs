use crate::fst_parser_type::ParseType;

use indexmap::IndexSet;
use crate::book::Book;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

#[derive(Default)]
pub struct ParseItemResult {
    pub item_links : Vec<usize>
}

impl ParseItemResult {
    pub fn add(&mut self, parse_result:&mut ParseResult, parse_type: ParseType, data: String) {
        let data_idx = parse_result.data[parse_type as usize].insert_full(data);
        self.item_links.push(data_idx.0);
    }
    pub fn reset(&mut self) {
        self.item_links.clear();
    }
}

#[derive(Default)]
pub struct ParseResult {
    pub books : Vec<Book>,
    pub data : Vec<IndexSet<String>>,
}

pub trait FSTParser {
    fn text(&mut self, text: &str, parse_result:&mut ParseResult);
    fn reset(&mut self);
    fn start_node(&mut self, text: &str);
    fn attribute(&mut self, attribute_name: &str, attribute_value: &str, parse_result:&mut ParseResult);
    fn end_node(&mut self, node_name: &str);
    fn is_found(&self) -> bool;
    fn has_results(&self) -> bool;
    fn get_parse_type(&self) -> ParseType;
    fn get_result(&self) -> Result<&ParseItemResult, ParseError>;
}