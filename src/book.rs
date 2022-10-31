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

    fn get_str_single(id: usize, parse_type: ParseType, parse_result: &ParseResult) -> String {
        parse_result.data[parse_type as usize][id].clone()
    }

    pub fn debug(&self, parse_result:&ParseResult) {
        if(self.language_ids.len() > 1) {
        println!("---------BOOK {} -------", self.gutenberg_book_id);
        println!("- title: {}", Book::get_str_single(self.title_id, ParseType::Title, parse_result));
        println!("- publisher: {}", Book::get_str_single(self.publisher_id, ParseType::Publisher, parse_result));
        println!("- rights: {}", Book::get_str_single(self.rights_id, ParseType::Rights, parse_result));
        println!("- date issued: {}", self.date_issued);
        println!("- number of downloads: {}", self.num_downloads);
        println!("- languages: {}", Book::get_str(&self.language_ids, ParseType::Language, parse_result));
        println!("- bookshelves: {}", Book::get_str(&self.bookshelf_ids, ParseType::Bookshelf, parse_result));
        println!("- subjects: {}", Book::get_str(&self.subject_ids, ParseType::Subject, parse_result));
        println!("- authors: {}", Book::get_str(&self.author_ids, ParseType::Author, parse_result));
        }
    }
}