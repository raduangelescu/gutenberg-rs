use crate::book::GutenbergFileEntry;
use crate::error::Error;
use crate::fst_parser::{FSTParser, ParseItemResult, ParseResult};
use crate::fst_parser_node::FSTParserNode;
use crate::fst_parser_type::ParseType;
use std::str;

pub struct FSTParserOrNode {
    pub nodes: Vec<FSTParserNode>,
    pub parse_type: ParseType,
}

impl FSTParser for FSTParserOrNode {
    fn text(
        &mut self,
        text: &str,
        parse_result: &mut ParseResult,
        book_id: i32,
    ) -> Result<(), Error> {
        for node in &mut self.nodes {
            node.text(text, parse_result, book_id)?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        for node in &mut self.nodes {
            node.reset();
        }
    }

    fn attribute(
        &mut self,
        _attribute_name: &str,
        _attribute_value: &str,
        _parse_result: &mut ParseResult,
        _book_id: i32,
    ) {
    }

    fn start_node(&mut self, node_name: &str) {
        for node in &mut self.nodes {
            node.start_node(node_name);
        }
    }

    fn end_node(&mut self, node_name: &str) {
        for node in &mut self.nodes {
            node.end_node(node_name);
        }
    }

    fn is_found(&self) -> bool {
        for node in &self.nodes {
            if node.is_found() {
                return true;
            }
        }
        return false;
    }

    fn has_results(&self) -> bool {
        for node in &self.nodes {
            if node.has_results() {
                return true;
            }
        }
        return false;
    }

    fn get_parse_type(&self) -> ParseType {
        self.parse_type
    }

    fn get_result(&self) -> Result<&ParseItemResult, Error> {
        for node in &self.nodes {
            if node.has_results() {
                return node.get_result();
            }
        }
        Err(Error::InvalidResult("no results".to_string()))
    }
    
    fn get_files(&self) -> Result<Vec<GutenbergFileEntry>, Error> {
        Err(Error::InvalidResult("no files".to_string()))
    }
}

impl FSTParserOrNode {
    pub fn build(states_str: Vec<Vec<String>>, parse_type: ParseType) -> Box<dyn FSTParser> {
        let mut nodes = Vec::new();
        for node_states in states_str {
            nodes.push(FSTParserNode {
                pos: -1,
                states: node_states,
                has_result: false,
                parse_type,
                result: Default::default(),
            })
        }
        return Box::new(FSTParserOrNode { nodes, parse_type });
    }
}
