use crate::book::GutenbergFileEntry;
use crate::error::Error;
use crate::fst_parser::{FSTParser, ParseItemResult, ParseResult};
use crate::fst_parser_type::ParseType;

use std::str;

pub struct FSTParserNode {
    pub pos: i32,
    pub states: Vec<String>,
    pub result: ParseItemResult,
    pub has_result: bool,
    pub parse_type: ParseType,
}

impl FSTParser for FSTParserNode {
    fn text(
        &mut self,
        text: &str,
        parse_result: &mut ParseResult,
        book_id: i32,
    ) -> Result<(), Error> {
        if !self.is_found() {
            return Ok(());
        }
        self.has_result = true;

        self.result
            .add(parse_result, self.parse_type, text.to_string(), book_id)?;
        Ok(())
    }

    fn reset(&mut self) {
        self.has_result = false;
        self.pos = -1;
        self.result.reset();
    }

    fn attribute(
        &mut self,
        _attribute_name: &str,
        _attribute_value: &str,
        _parse_result: &mut ParseResult,
        _book_id: i32,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn start_node(&mut self, node_name: &str) {
        if self.pos == -1 && node_name == self.states[0] {
            self.pos = 0;
            return;
        }
        if self.pos != -1 {
            let check_index = self.pos + 1;
            if check_index >= self.states.len() as i32 {
                return;
            }
            if node_name == self.states[check_index as usize] {
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

    fn get_result(&self) -> Result<&ParseItemResult, Error> {
        Ok(&self.result)
    }

    fn get_files(&self) -> Result<Vec<GutenbergFileEntry>, Error> {
        Err(Error::InvalidResult("no results".to_string()))
    }
}

impl FSTParserNode {
    pub fn build(path: &'static str, parse_type: ParseType) -> Box<dyn FSTParser> {
        let states: Vec<String> = path.split("/").map(|s| String::from(s)).collect();
        Box::new(FSTParserNode {
            pos: -1,
            states,
            has_result: false,
            parse_type,
            result: Default::default(),
        })
    }
}
