use crate::fst_parser_type::ParseType;
use crate::fst_parser::FSTParser;

use fast_xml::events::attributes::Attributes;
use std::borrow::Borrow;
use std::str;

pub struct FSTParserNode {
    pub pos: i32,
    pub states: Vec<String>,
    pub results: Vec<String>,
    pub has_result: bool,
    pub parse_type: ParseType,
    pub attribute: String,
}

impl FSTParser for FSTParserNode {
    fn text(&mut self, text: &str) {
        if self.is_found() && self.attribute.eq("") {
            self.has_result = true;
            self.results.push(String::from(text));
        }
    }

    fn reset(&mut self) {
        self.has_result = false;
        self.results = vec![];
        self.pos = -1;
    }

    fn start_node(&mut self, node_name: &str, attributes: Attributes) {
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
        if self.is_found() && !self.attribute.eq("") {
            for attr in attributes {
                let attr_val = attr.unwrap();
                if attr_val.key.eq(self.attribute.as_bytes()) {
                    let value = str::from_utf8(attr_val.value.borrow()).unwrap();
                    self.results.push(String::from(value));
                    self.has_result = true;
                    break;
                }
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
    fn get_results(&self) -> &Vec<String> {
        &self.results
    }
}

impl FSTParserNode {
    pub fn build(states_str: Vec<&str>, parse_type: ParseType, attribute: &str) -> FSTParserNode {
        let states = states_str.iter().map(|&v| String::from(v)).collect();
        FSTParserNode {
            pos: -1,
            states,
            has_result: false,
            results: vec![],
            parse_type,
            attribute: String::from(attribute),
        }
    }
}

