use crate::fst_parser_type::ParseType;
use crate::fst_parser::{FSTParser, ParseResult, ParseItemResult};
use crate::error::{ParseError};

use std::str;

pub(crate) struct FSTParserFileNode {
    pos: i32,
    states: Vec<String>,
    attribute: String,
    result_files: ParseItemResult,
    result_file_links: ParseItemResult,
    has_node: bool,
    parse_type: ParseType,
}

impl FSTParser for FSTParserFileNode {
    fn text(&mut self, text: &str, parse_result:&mut ParseResult, book_id: i32) {
        if self.is_found() {
            self.has_node = true;
            self.result_files.add(parse_result, self.parse_type, text.to_string(), book_id);
        }
    }

    fn reset(&mut self) {
        self.has_node = false;
        self.pos = -1;
        self.result_file_links.reset();
        self.result_files.reset();
    }

    fn attribute(&mut self, attribute_name: &str, attribute_value: &str, parse_result:&mut ParseResult, book_id: i32) {
            if !self.is_found() {
                return;
            }
        
            if attribute_name != self.attribute {
                return;
            }

            let value = attribute_value;
            self.result_file_links.add(parse_result, self.parse_type, value.to_string(), book_id);
    }
    

    fn start_node(&mut self, node_name: &str) {
        if self.pos == -1 && node_name.eq(&self.states[0]) {
            self.pos = 0;
            return;
        }

        if self.pos != -1 {
            let check_index = self.pos + 1;
            if check_index >= self.states.len() as i32 {
                return;
            }
            if node_name.eq(&self.states[check_index as usize]) {
                self.pos += 1;
            }
        }
    }

    fn end_node(&mut self, node_name: &str) {
        if self.pos > -1 {
            if self.states[self.pos as usize].eq(node_name) {
                self.pos -= 1;
            }
        }
    }

    fn is_found(&self) -> bool {
        self.pos == self.states.len() as i32 - 1
    }

    fn has_results(&self) -> bool {
        return self.has_node && !self.result_file_links.item_links.is_empty();
    }

    fn get_parse_type(&self) -> ParseType {
        self.parse_type
    }

    fn get_result(&self) -> Result<&ParseItemResult, ParseError> {
        Ok(&self.result_files)
    }
    
}

impl FSTParserFileNode {
    pub fn build(states_str: Vec<&str>, attribute:&str, parse_type: ParseType) -> Box<dyn FSTParser> {
        Box::new(FSTParserFileNode {
            pos: -1,
            states: states_str.iter().map(|&v| String::from(v)).collect(),
            has_node: false,
            parse_type,
            result_files : Default::default(),
            result_file_links : Default::default(),
            attribute: attribute.to_string(),
        })
    }
}

