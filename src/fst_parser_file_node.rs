use crate::book::GutenbergFileEntry;
use crate::error::Error;
use crate::fst_parser::{FSTParser, ParseItemResult, ParseResult};
use crate::fst_parser_type::ParseType;

use std::str;

pub(crate) struct FSTParserFileNode {
    pos: i32,
    attribute_states_idx: i32,
    states: Vec<String>,
    attribute: String,
    has_node: bool,
    parse_type: ParseType,

    files: Vec<GutenbergFileEntry>,
}

impl FSTParser for FSTParserFileNode {
    fn text(
        &mut self,
        text: &str,
        parse_result: &mut ParseResult,
        book_id: i32,
    ) -> Result<(), Error> {
        if !self.is_found() {
            return Err(Error::InvalidRdf("No files".to_string()));
        }
        self.has_node = true;
        let idx = parse_result.add_file_type(text.to_string(), book_id)?;
        if let Some(last_file) = self.files.last_mut() { 
            last_file.file_type_id = (idx + 1) as i32;
            return Ok(());
        }
        Err(Error::InvalidRdf("No files".to_string()))
    }

    fn reset(&mut self) {
        self.has_node = false;
        self.pos = -1;
        self.files.clear();
    }

    fn attribute(
        &mut self,
        attribute_name: &str,
        attribute_value: &str,
        parse_result: &mut ParseResult,
        book_id: i32,
    ) -> Result<(), Error> {
        if self.attribute_states_idx != self.pos {
            return Ok(());
        }

        if attribute_name != self.attribute {
            return Ok(());
        }
        
        let value = attribute_value;
        let file_link_id = parse_result.add_file(value.to_string(), book_id)?;
   
        self.files.push(GutenbergFileEntry {
            file_link_id: file_link_id as i32,
            file_type_id: -1,
        });
        Ok(())
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
        return self.has_node && !self.files.is_empty();
    }

    fn get_parse_type(&self) -> ParseType {
        self.parse_type
    }

    fn get_result(&self) -> Result<&ParseItemResult, Error> {
        Err(Error::InvalidResult("no result in file parser".to_string()))
    }

    fn get_files(&self) -> Result<Vec<GutenbergFileEntry>, Error> {
        Ok(self.files.to_vec())
    }
}

impl FSTParserFileNode {
    pub fn build(
        states_str: Vec<&str>,
        attribute: &str,
        parse_type: ParseType,
    ) -> Box<dyn FSTParser> {
        Box::new(FSTParserFileNode {
            pos: -1,
            states: states_str.iter().map(|&v| String::from(v)).collect(),
            has_node: false,
            parse_type,
            attribute: attribute.to_string(),
            attribute_states_idx: 1,
            files: Vec::new(),
        })
    }
}
