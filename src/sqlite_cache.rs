use crate::error::Error;
use crate::fst_parser::DictionaryItemContent;
use crate::fst_parser::ParseResult;
use crate::fst_parser_type::ParseType;
use crate::settings::GutenbergCacheSettings;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use num_traits::FromPrimitive;
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct SQLiteCache {
    pub connection: Box<Connection>,
}
struct HelperQuery<'a> {
    tables: Vec<&'a str>,
    query_struct: Vec<&'a str>,
}

impl SQLiteCache {
    pub fn get_download_links(&mut self, ids: Vec<i32>) -> Result<Vec<String>, Error> {
        let ids_collect = ids.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let ids_str = ids_collect.join(",");
        let q = format!("SELECT downloadlinks.name FROM downloadlinks, books WHERE downloadlinks.bookid = books.id AND books.gutenbergbookid IN ({}) AND downloadlinks.downloadtypeid in (5, 6, 10,13,26,27,28,33,34,35,40,46,49,51)", ids_str);
        let mut stmt = self.connection.prepare(&q)?;
        let mut rows = stmt.query(())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(row.get(0)?);
        }
        Ok(results)
    }

    pub fn query(&mut self, json: &Value) -> Result<Vec<i32>, Error> {
        let mut helpers = Vec::new();

        if let Some(field) = json.get("language") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["languages", "book_languages"],
                    query_struct: vec![
                        "languages.id = book_languages.languageid AND books.id = book_languages.bookid",
                        "languages.name",
                        field_value,
                    ],
                });
            } else {
                return Err(Error::InvalidQuery("language must be a string".to_string()));
            }
        }
        if let Some(field) = json.get("author") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["authors", "book_authors"],
                    query_struct: vec![
                        "authors.id = book_authors.authorid and books.id = book_authors.bookid",
                        "authors.name",
                        field_value,
                    ],
                });
            } else {
                return Err(Error::InvalidQuery("author must be a string".to_string()));
            }
        }
        if let Some(field) = json.get("title") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["titles"],
                    query_struct: vec!["titles.bookid = books.id", "titles.name", field_value],
                });
            } else {
                return Err(Error::InvalidQuery("title must a string".to_string()));
            }
        }
        if let Some(field) = json.get("subject") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["subjects", "book_subjects"],
                    query_struct: vec![
                        "subjects.id = book_subjects.bookid and books.id = book_subjects.subjectid ",
                        "subjects.name",
                        field_value,
                    ],
                });
            } else {
                return Err(Error::InvalidQuery("subject must a string".to_string()));
            }
        }
        if let Some(field) = json.get("publisher") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["publishers"],
                    query_struct: vec![
                        "publishers.id = books.publisherid",
                        "publishers.name",
                        field_value,
                    ],
                });
            } else {
                return Err(Error::InvalidQuery("publisher must a string".to_string()));
            }
        }
        if let Some(field) = json.get("bookshelve") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["bookshelves, book_bookshelves"],
                    query_struct: vec![
                        "bookshelves.id = book_bookshelves.bookshelfid AND books.id = book_bookshelves.bookid",
                        "bookshelves.name",
                        field_value,
                    ],
                });
            } else {
                return Err(Error::InvalidQuery("bookshelve must a string".to_string()));
            }
        }
        if let Some(field) = json.get("rights") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery {
                    tables: vec!["rights"],
                    query_struct: vec!["rights.id = books.rightsid", "rights.name", field_value],
                });
            } else {
                return Err(Error::InvalidQuery("rights must be a string".to_string()));
            }
        }
        if let Some(field) = json.get("downloadlinkstype") {
            if let Some(field_value) = field.as_str() {
                helpers.push(HelperQuery{tables: vec!["downloadlinks", "downloadlinkstype"],
                            query_struct: vec!["downloadlinks.downloadtypeid =  downloadlinkstype.id and downloadlinks.bookid = books.id",
                            "downloadlinkstype.name",
                            field_value]});
            } else {
                return Err(Error::InvalidQuery(
                    "downloadlinkstype must a string".to_string(),
                ));
            }
        }

        let mut query = "SELECT DISTINCT books.gutenbergbookid FROM books".to_string();
        for q in &helpers {
            query = format!("{},{}", query, q.tables.join(","))
        }

        query = format!("{} WHERE ", query);
        let mut idx = 0;
        for q in &helpers {
            query = format!(
                "{} {} and {} in ({}) ",
                query, q.query_struct[0], q.query_struct[1], q.query_struct[2]
            );
            if idx != helpers.len() - 1 {
                query = format!("{} and ", query);
            }
            idx = idx + 1;
        }
        let mut stmt = self.connection.prepare(&query)?;
        let mut rows = stmt.query(())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(row.get(0)?);
        }

        Ok(results)
    }
}

impl SQLiteCache {
    pub fn get_cache(settings: &GutenbergCacheSettings) -> Result<SQLiteCache, Error> {
        if Path::new(&settings.cache_filename).exists() {
            let connection = Box::new(Connection::open(&settings.cache_filename)?);
            return Ok(SQLiteCache { connection });
        }
        Err(Error::InvalidIO(
            format!("No cache file {}", settings.cache_filename).to_string(),
        ))
    }

    pub fn create_cache(
        parse_results: &ParseResult,
        settings: &GutenbergCacheSettings,
        force_recreate: bool,
        show_progress_bar: bool,
    ) -> Result<SQLiteCache, Error> {
        if Path::new(&settings.cache_filename).exists() && !settings.db_in_memory {
            if force_recreate {
                fs::remove_file(&settings.cache_filename)?;
            } else {
                let connection = Box::new(Connection::open(&settings.cache_filename)?);
                return Ok(SQLiteCache { connection });
            }
        }
        let mut connection = match settings.db_in_memory {
            false => Box::new(Connection::open(&settings.cache_filename)?),
            true => Box::new(Connection::open(":memory:")?),
        };
        let create_query = include_str!("gutenbergindex.db.sql");
        connection.execute_batch(create_query)?;
        connection.execute_batch("PRAGMA journal_mode = OFF;PRAGMA synchronous = 0;PRAGMA cache_size = 1000000;PRAGMA locking_mode = EXCLUSIVE;PRAGMA temp_store = MEMORY;")?;

        let mut book_id = 0;

        let mut pb_fields: Option<ProgressBar> = None;
        if show_progress_bar {
            let pb = ProgressBar::new(parse_results.field_dictionaries.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})",
                )?
                .progress_chars("???  "),
            );
            pb_fields = Some(pb);
        }

        for (idx, result) in parse_results.field_dictionaries.iter().enumerate() {
            book_id = book_id + 1;
            if let Some(pb) = &mut pb_fields {
                pb.set_position((idx + 1) as u64);
            }

            match FromPrimitive::from_usize(idx) {
                Some(ParseType::Title) => {
                    SQLiteCache::insert_many_field_id(
                        &mut connection,
                        "titles",
                        "name",
                        "bookid",
                        result,
                        book_id,
                    )?;
                }
                Some(ParseType::Subject) => {
                    SQLiteCache::insert_many_fields(&mut connection, "subjects", "name", result)?;
                }
                Some(ParseType::Language) => {
                    SQLiteCache::insert_many_fields(&mut connection, "languages", "name", result)?;
                }
                Some(ParseType::Author) => {
                    SQLiteCache::insert_many_fields(&mut connection, "authors", "name", result)?;
                }
                Some(ParseType::Bookshelf) => {
                    SQLiteCache::insert_many_fields(
                        &mut connection,
                        "bookshelves",
                        "name",
                        result,
                    )?;
                }
                Some(ParseType::Publisher) => {
                    SQLiteCache::insert_many_fields(&mut connection, "publishers", "name", result)?;
                }
                Some(ParseType::Rights) => {
                    SQLiteCache::insert_many_fields(&mut connection, "rights", "name", result)?;
                }
                _ => {}
            }
        }
        if let Some(pb) = pb_fields {
            pb.finish();
        }
        SQLiteCache::insert_many_fields(
            &mut connection,
            "downloadlinkstype",
            "name",
            &parse_results.file_types_dictionary,
        )?;
        let mut pb_all: Option<ProgressBar> = None;
        if show_progress_bar {
            let pb = ProgressBar::new(parse_results.books.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})",
                )?
                .progress_chars("???  "),
            );

            pb.set_message(format!("Building sqlite db"));
            pb_all = Some(pb);
        }

        for (idx, book) in parse_results.books.iter().enumerate() {
            if let Some(pb) = &mut pb_all {
                pb.set_position(idx as u64);
            }
            let pairs_book_authors = book
                .author_ids
                .iter()
                .map(|x| (*x + 1, idx + 1))
                .collect::<Vec<(usize, usize)>>();
            let pairs_book_subjects = book
                .subject_ids
                .iter()
                .map(|x| (*x + 1, idx + 1))
                .collect::<Vec<(usize, usize)>>();
            let pairs_book_languages = book
                .language_ids
                .iter()
                .map(|x| (*x + 1, idx + 1))
                .collect::<Vec<(usize, usize)>>();
            let pairs_book_bookshelves = book
                .bookshelf_ids
                .iter()
                .map(|x| (*x + 1, idx + 1))
                .collect::<Vec<(usize, usize)>>();

            SQLiteCache::insert_links(
                &mut connection,
                pairs_book_authors,
                "book_authors",
                "authorid",
                "bookid",
            )?;
            SQLiteCache::insert_links(
                &mut connection,
                pairs_book_subjects,
                "book_subjects",
                "subjectid",
                "bookid",
            )?;
            SQLiteCache::insert_links(
                &mut connection,
                pairs_book_languages,
                "book_languages",
                "languageid",
                "bookid",
            )?;
            SQLiteCache::insert_links(
                &mut connection,
                pairs_book_bookshelves,
                "book_bookshelves",
                "bookshelfid",
                "bookid",
            )?;

            let query = format!(
                "INSERT OR IGNORE INTO downloadlinks(name, downloadtypeid, bookid) VALUES (?,?,?)"
            );

            let mut smt = connection.prepare(query.as_str())?;
            for item in book.files.iter() {
                let mut file_link = "";
                if let Some(file_link_item) = parse_results
                    .files_dictionary
                    .get_index(item.file_link_id as usize)
                {
                    file_link = file_link_item.0;
                }
                smt.execute([
                    file_link,
                    item.file_type_id.to_string().as_str(),
                    (idx + 1).to_string().as_str(),
                ])?;
            }

            connection.execute("INSERT OR IGNORE INTO books(publisherid,rightsid,numdownloads,gutenbergbookid) VALUES (?,?,?,?)"
            , (book.publisher_id, book.rights_id,
            book.num_downloads,book.gutenberg_book_id))?;
        }
        let create_query = include_str!("gutenbergindex_indices.db.sql");
        connection.execute_batch(create_query)?;

        if let Some(pb) = pb_all {
            pb.finish();
        }

        Ok(SQLiteCache { connection })
    }

    fn insert_links(
        connection: &mut Connection,
        links: Vec<(usize, usize)>,
        table_name: &str,
        link1_name: &str,
        link2_name: &str,
    ) -> Result<(), Error> {
        if links.is_empty() {
            return Ok(());
        }
        let query = std::format!(
            "INSERT INTO {}({},{}) VALUES (?,?)",
            table_name,
            link1_name,
            link2_name
        );

        let mut smt = connection.prepare(query.as_str())?;
        for item in links.iter() {
            smt.execute([item.0.to_string(), item.1.to_string()])?;
        }
        Ok(())
    }
    fn insert_many_fields(
        connection: &mut Connection,
        table: &str,
        field: &str,
        field_dictionary: &IndexMap<String, DictionaryItemContent>,
    ) -> Result<(), Error> {
        if field_dictionary.is_empty() {
            return Ok(());
        }

        let query = std::format!("INSERT OR IGNORE INTO {}({}) VALUES(?)", table, field);

        let mut smt = connection.prepare(query.as_str())?;
        for item in field_dictionary.iter() {
            smt.execute([item.0.as_str()])?;
        }
        Ok(())
    }

    fn insert_many_field_id(
        connection: &mut Connection,
        table: &str,
        field1: &str,
        field2: &str,
        field_dictionary: &IndexMap<String, DictionaryItemContent>,
        _book_id: usize,
    ) -> Result<(), Error> {
        if field_dictionary.is_empty() {
            return Ok(());
        }

        let query = format!(
            "INSERT OR IGNORE INTO {}({}, {}) VALUES (?,?)",
            table, field1, field2
        );

        let mut smt = connection.prepare(query.as_str())?;
        for item in field_dictionary.iter() {
            for book_id in &item.1.book_links {
                smt.execute([item.0.as_str(), book_id.to_string().as_str()])?;
            }
        }
        Ok(())
    }
}
