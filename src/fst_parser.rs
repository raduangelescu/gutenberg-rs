use crate::fst_parser_type::ParseType;
use crate::error::ParseError;
use indexmap::{IndexMap};
use crate::book::{Book,GutenbergFileEntry};
use std::error::Error;

#[derive(Default)]
pub struct ParseItemResult {
    pub item_links : Vec<usize>
}

impl ParseItemResult {
    pub fn add(&mut self, parse_result:&mut ParseResult, parse_type: ParseType, data: String, book_id: i32) -> Result<(), Box<dyn Error>> {
        let data_idx = parse_result.add_field(parse_type, data, book_id)?;
        self.item_links.push(data_idx);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.item_links.clear();
    }
}

#[derive(Default)]
pub struct DictionaryItemContent {
    pub book_links : Vec<usize>,
}

#[derive(Default)]
pub struct ParseResult {
    pub books : Vec<Book>,
    pub field_dictionaries : Vec<IndexMap<String, DictionaryItemContent>>,
    pub files_dictionary : IndexMap<String, DictionaryItemContent>,
    pub file_types_dictionary : IndexMap<String, DictionaryItemContent>,
}

impl ParseResult {
    pub fn add(map :&mut IndexMap<String, DictionaryItemContent>, data: String, book_id : i32) -> Result<usize, ParseError>{
        let map_entry = map.get_full_mut(data.as_str());
        if map_entry.is_none() {
            let result = map.insert_full(data, DictionaryItemContent {
                book_links : vec![book_id as usize],
            });
            return Ok(result.0);
        }
        match map_entry {
            Some(data) => { 
                data.2.book_links.push(book_id as usize);
                return Ok(data.0);
            }
            None => { return Err(ParseError::InvalidResult("bad data".to_string()));}
        }
    }
    pub fn add_file(&mut self, data: String, book_id : i32) -> Result<usize, ParseError> {
        ParseResult::add(&mut self.files_dictionary, data, book_id)
    }
    pub fn add_file_type(&mut self, data: String, book_id : i32) -> Result<usize, ParseError> {
        ParseResult::add(&mut self.file_types_dictionary, data, book_id)
    }
    pub fn add_field(&mut self, field: ParseType, data: String, book_id : i32) -> Result<usize, ParseError> { 
        ParseResult::add(&mut self.field_dictionaries[field as usize], data, book_id)
    }
}
pub trait FSTParser {
    fn text(&mut self, text: &str, parse_result:&mut ParseResult, book_id: i32)-> Result<(), Box<dyn Error>>;
    fn reset(&mut self);
    fn start_node(&mut self, text: &str);
    fn attribute(&mut self, attribute_name: &str, attribute_value: &str, parse_result:&mut ParseResult, book_id: i32);
    fn end_node(&mut self, node_name: &str);
    fn is_found(&self) -> bool;
    fn has_results(&self) -> bool;
    fn get_parse_type(&self) -> ParseType;
    fn get_result(&self) -> Result<&ParseItemResult, ParseError>;
    fn get_files(&self) ->  Result<Vec<GutenbergFileEntry>, ParseError>;
}