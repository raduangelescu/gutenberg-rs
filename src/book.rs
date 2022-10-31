use crate::fst_parser::ParseResult;
use crate::fst_parser_type::ParseType;

pub struct Book {
    pub publisher_id : usize,
    pub title_id : usize,
    pub rights_id : usize,
    pub gutenberg_book_id : usize,
    pub date_issued : String,
    pub num_downloads : i32,

    pub language_ids : Vec<usize>,
    pub subject_ids : Vec<usize>,
    pub author_ids : Vec<usize>,
    pub bookshelf_ids : Vec<usize>,
}

impl Book {
    fn get_str(ids: &Vec<usize>, parse_type: ParseType, parse_result: &ParseResult) -> String {
        ids
        .iter()
        .map(|x|  parse_result.data[parse_type as usize][*x].clone())
        .collect::<Vec<_>>()
        .join("||")
    }

    pub fn debug(&self, parse_result:&ParseResult) {
        println!("---------BOOK {} -------", self.gutenberg_book_id);
        println!("- languages: {}", Book::get_str(&self.language_ids, ParseType::Language, parse_result));
        println!("- bookshelves: {}", Book::get_str(&self.bookshelf_ids, ParseType::Bookshelf, parse_result));
        println!("- subjects: {}", Book::get_str(&self.subject_ids, ParseType::Subject, parse_result));
        println!("- authors: {}", Book::get_str(&self.author_ids, ParseType::Author, parse_result));
        
    }
}