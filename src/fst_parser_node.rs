use crate::fst_parser_type::ParseType;
use crate::fst_parser::{FSTParser, ParseItemResult, ParseResult, ParseError};

use std::str;

pub struct FSTParserNode {
    pub pos: i32,
    pub states: Vec<String>,
    pub result: ParseItemResult,
    pub has_result: bool,
    pub parse_type: ParseType,
}

impl FSTParser for FSTParserNode {
    fn text(&mut self, text: &str, parse_result:&mut ParseResult) {
        if self.is_found() {
            self.has_result = true;
            self.result.add(parse_result, self.parse_type, text.to_string());
        }
    }

    fn reset(&mut self) {
        self.has_result = false;
        self.pos = -1;
        self.result.reset();
    }

    fn attribute(&mut self, attribute_name: &str, attribute_value: &str, parse_result:&mut ParseResult) {}

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
        return self.has_result;
    }
    fn get_parse_type(&self) -> ParseType {
        self.parse_type
    }

    fn get_result(&self) -> Result<&ParseItemResult, ParseError> {
        Ok(&self.result)
    }
}

impl FSTParserNode {
    pub fn build(states_str: Vec<&str>, parse_type: ParseType) -> Box<dyn FSTParser> {
        let states = states_str.iter().map(|&v| String::from(v)).collect();
        Box::new(FSTParserNode {
            pos: -1,
            states,
            has_result: false,
            parse_type,
            result : Default::default(),
        })
    }
}

