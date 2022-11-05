use crate::db_cache::DBCache;
use crate::fst_parser_type::ParseType;
use crate::fst_parser::ParseResult;
use crate::fst_parser::DictionaryItemContent;
use num_traits::FromPrimitive;
use indexmap::IndexMap;
use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};

pub struct SQLiteCache {
    sqlite_db_filename : String,
}

impl Default for SQLiteCache {
    fn default() -> SQLiteCache {
    
        SQLiteCache {
            sqlite_db_filename : String::from("gutenberg-rs.db"),
        }
    }
}

impl DBCache for SQLiteCache {
    fn create_cache(&mut self, parse_results: &ParseResult) ->Result<(), Box<dyn Error>> {
        if Path::new(&self.sqlite_db_filename).exists() {
            fs::remove_file(&self.sqlite_db_filename)?;
        }
        let mut connection = Connection::open(&self.sqlite_db_filename)?;
        let create_query = include_str!("gutenbergindex.db.sql");
        connection.execute_batch(create_query)?;
        connection.execute_batch("PRAGMA journal_mode = OFF;PRAGMA synchronous = 0;PRAGMA cache_size = 1000000;PRAGMA locking_mode = EXCLUSIVE;PRAGMA temp_store = MEMORY;")?;
        let mut book_id = 0;
        let pb_fields = ProgressBar::new(parse_results.field_dictionaries.len() as u64);
        pb_fields.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})")
        ?.progress_chars("█  "));

        for (idx,  result) in parse_results.field_dictionaries.iter().enumerate() {
            book_id = book_id + 1;
            pb_fields.set_position((idx + 1) as u64);
            
            match FromPrimitive::from_usize(idx) {
                Some(ParseType::Title) => {
                    pb_fields.set_message("Fields titles");
                    SQLiteCache::insert_many_field_id(&mut connection,
                        "titles",
                        "name",
                        "bookid",
                        result,
                        book_id)?;
                        
                    }
                Some(ParseType::Subject) => {
                    pb_fields.set_message("Fields subjects");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "subjects",
                        "name",
                    result)?;
                    
                },
                Some(ParseType::Language) => {
                    pb_fields.set_message("Fields language");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "languages",
                        "name",
                    result)?;
                },
                Some(ParseType::Author) => {
                    pb_fields.set_message("Fields author");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "authors",
                        "name",
                    result)?;
                },
                Some(ParseType::Bookshelf) => {
                    pb_fields.set_message("Fields bookshelf");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "bookshelves",
                        "name",
                    result)?;
                },
                Some(ParseType::Files) => {},
                Some(ParseType::Publisher) => {
                    pb_fields.set_message("Fields publisher");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "publishers",
                        "name",
                    result)?;
                },
                Some(ParseType::Rights) => {
                    pb_fields.set_message("Fields rights");
                    SQLiteCache::insert_many_fields(&mut connection,
                        "rights",
                        "name",
                    result)?;
                },
                Some(ParseType::DateIssued) => {},
                Some(ParseType::Downloads) => {},
                None => {},
            }
            
        }
        pb_fields.finish();
        SQLiteCache::insert_many_fields(&mut connection,
            "downloadlinkstype",
            "name", &parse_results.file_types_dictionary)?;
        
        let pb = ProgressBar::new(parse_results.books.len() as u64);
        pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})")
        ?.progress_chars("█  "));
        
        pb.set_message(format!("Building sqlite db"));

        for (idx, book) in parse_results.books.iter().enumerate() {
            pb.set_position(idx as u64);
            let pairs_book_authors = book.author_ids.iter().map(|x| (*x + 1, idx + 1)).collect::<Vec<(usize, usize)>>();
            let pairs_book_subjects = book.subject_ids.iter().map(|x| (*x + 1, idx + 1)).collect::<Vec<(usize, usize)>>();
            let pairs_book_languages = book.language_ids.iter().map(|x| (*x + 1, idx + 1)).collect::<Vec<(usize, usize)>>();
            let pairs_book_bookshelves = book.bookshelf_ids.iter().map(|x| (*x + 1, idx + 1)).collect::<Vec<(usize, usize)>>();
            
            SQLiteCache::insert_links(&mut connection, pairs_book_authors, "book_authors", "authorid", "bookid")?;
            SQLiteCache::insert_links(&mut connection, pairs_book_subjects, "book_subjects", "subjectid", "bookid")?;
            SQLiteCache::insert_links(&mut connection, pairs_book_languages, "book_languages", "languageid", "bookid")?;
            SQLiteCache::insert_links(&mut connection, pairs_book_bookshelves, "book_bookshelves", "bookshelfid", "bookid")?;
            
            let query = format!("INSERT OR IGNORE INTO downloadlinks(name, downloadtypeid, bookid) VALUES (?,?,?)");
        
            let mut smt = connection.prepare(query.as_str())?;
            for item in book.files.iter() {
                let file_link = parse_results.files_dictionary.get_index(item.file_link_id as usize).unwrap().0;
                smt.execute([file_link, item.file_type_id.to_string().as_str(), idx.to_string().as_str()])?;
            }
        
            connection.execute("INSERT OR IGNORE INTO books(publisherid,dateissued,rightsid,numdownloads,gutenbergbookid) VALUES (?,?,?,?,?)"
            , (book.publisher_id, book.date_issued.clone(), book.rights_id,
            book.num_downloads,book.gutenberg_book_id))?;
        }
        let create_query = include_str!("gutenbergindex_indices.db.sql");
        connection.execute_batch(create_query)?;
        
        pb.finish();
        Ok(())
    }
    fn query(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn native_query(self, __query: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl SQLiteCache {
    fn insert_links (connection: &mut Connection, links: Vec<(usize, usize)>, table_name: &str, link1_name: &str, link2_name: &str) -> Result<(), Box<dyn Error>>{
        if links.is_empty() {
            return Ok(())
        }
        let query = std::format!("INSERT INTO {}({},{}) VALUES (?,?)", table_name, link1_name, link2_name);

        let mut smt = connection.prepare(query.as_str())?;
        for item in links.iter() {
            smt.execute([item.0.to_string(), item.1.to_string()])?;
        }
        Ok(())
    }
    fn insert_many_fields(connection: &mut Connection, table: &str, field: &str, field_dictionary: &IndexMap<String, DictionaryItemContent>) -> Result<(), Box<dyn Error>>{
        if field_dictionary.is_empty() {
            return Ok(())
        }

        let query = std::format!("INSERT OR IGNORE INTO {}({}) VALUES(?)", table, field);

        let mut smt = connection.prepare(query.as_str())?;
        for item in field_dictionary.iter() {
            smt.execute([item.0.as_str().as_bytes()])?;
        }
        Ok(())
    }

    fn insert_many_field_id(connection: &mut Connection, table: &str, field1: &str, field2: &str, field_dictionary: &IndexMap<String, DictionaryItemContent>, _book_id: usize) -> Result<(), Box<dyn Error>> {
        if field_dictionary.is_empty() {
            return Ok(());
        }

        let query = format!("INSERT OR IGNORE INTO {}({}, {}) VALUES (?,?)", table, field1, field2);
        
        let mut smt = connection.prepare(query.as_str())?;
        for item in field_dictionary.iter() {
            for book_id in &item.1.book_links {
                smt.execute([item.0.as_str(), book_id.to_string().as_str()])?;
            }
        }
        Ok(())
    }
}