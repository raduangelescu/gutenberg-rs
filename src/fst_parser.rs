use fast_xml::events::attributes::Attributes;
use crate::fst_parser_type::ParseType;
pub trait FSTParser {
    fn text(&mut self, text: &str);
    fn reset(&mut self);
    fn start_node(&mut self, text: &str, attributes: Attributes);
    fn end_node(&mut self, node_name: &str);
    fn is_found(&self) -> bool;
    fn has_results(&self) -> bool;
    fn get_parse_type(&self) -> ParseType;
    fn get_results(&self) -> &Vec<String>;
}