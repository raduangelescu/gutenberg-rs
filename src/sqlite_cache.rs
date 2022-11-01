use crate::db_cache::DBCache;
use crate::fst_parser_type::ParseType;
use crate::fst_parser::ParseResult;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use indexmap::IndexSet;
use rusqlite::{Connection};
use std::fmt;

pub struct SQLiteCache {
    table_map : Vec<String>,
    sqlite_db_create_cache_filename : String,
    sqlite_db_create_indices_filename : String,
}

impl Default for SQLiteCache {
    fn default() -> SQLiteCache {
    
        SQLiteCache {
            table_map : vec!("title".to_string(),
                            "subject".to_string(),
                            "language".to_string(),
                            "author".to_string(),
                            "bookshelf".to_string(),
                            "/---/".to_string(),
                            "publishers".to_string(),
                            "rights".to_string()),
            sqlite_db_create_cache_filename : String::from("gutenbergindex.db.sql"),
            sqlite_db_create_indices_filename :  String::from("gutenbergindex_indices.db.sql"),
        }
    }
}

impl DBCache for SQLiteCache {
    fn create_cache(&mut self, parse_results: &ParseResult) { 
        let mut connection = Connection::open(&self.sqlite_db_create_cache_filename).unwrap();
        let create_query = include_str!("gutenbergindex.db.sql");
        connection.execute_batch(create_query).unwrap();
        
        for (idx,  result) in parse_results.data.iter().enumerate() {
            let book_id = parse_results.books[idx].gutenberg_book_id;
            match FromPrimitive::from_usize(idx) {
                Some(ParseType::Title) => {
                    SQLiteCache::insert_many_field_id(&mut connection,
                        "titles",
                        "name",
                        "book_id",
                        result, 
                        book_id);
                    }
                Some(ParseType::Subject) => {},
                Some(ParseType::Language) => {},
                Some(ParseType::Author) => {},
                Some(ParseType::Bookshelf) => {},
                Some(ParseType::Files) => {},
                Some(ParseType::Publisher) => {
                    SQLiteCache::insert_many_fields(&mut connection,
                        "publishers",
                        "name",
                    result);
                },
                Some(ParseType::Rights) => {
                    SQLiteCache::insert_many_fields(&mut connection,
                        "rights",
                        "name",
                    result);
                },
                Some(ParseType::DateIssued) => {},
                Some(ParseType::Downloads) => {},
                None => {},
            }
            /*if idx == ParseType::File as usize {
                self.insert_many_fields(connection, table, ordered_set, pt.setTypes)
                self.connection.executemany( "INSERT OR IGNORE INTO downloadlinks(name,bookid,downloadtypeid) VALUES (?,?,?)"
                , map(|x| => (x[0], x[1], x[2]) parse_results.field_sets[Fields.FILES].setLinks))
            }
            else if (result.needs_book_id()) {
                self.__insert_many_field_id(self.table_map[idx], "name", "bookid", pt.set)
            }
            else {
                self.__insert_many_field(self.table_map[idx], "name", pt.set)
            }*/
        }
    }
    fn query(&self) {

    }
    fn native_query(self, query: &str) {

    }
}
type OrderedSet = std::collections::BTreeSet<String>;

impl SQLiteCache {
    fn insert_many_fields(connection: &mut Connection, table: &str, field: &str, ordered_set: &IndexSet<String>) {
        if ordered_set.is_empty() {
            return;
        }

        let query = std::format!("INSERT OR IGNORE {}({}) VALUES(?)", table, field);

        let mut smt = connection.prepare(query.as_str()).unwrap();
        for item in ordered_set.iter() {
            smt.execute([item.as_str().as_bytes()]);
        }
        connection.flush_prepared_statement_cache();
    }

    fn insert_many_field_id(connection: &mut Connection, table: &str, field1: &str, field2: &str, ordered_set: &IndexSet<String>, book_id: usize) {
        if ordered_set.is_empty() {
            return;
        }

        let query = format!("INSERT OR IGNORE INTO {}({}, {}) VALUES (?,?)", table, field1, field2);
        
        let mut smt = connection.prepare(query.as_str()).unwrap();
        for item in ordered_set.iter() {
            smt.execute([book_id]);
            smt.execute([item.as_str().as_bytes()]);
        }
        connection.flush_prepared_statement_cache();
    }

    fn insert_links(connection: &mut Connection, ids: Vec<i32>, table_name: String, link1_name: &str, link2_name: &str) {
        if ids.is_empty() {
            return;
        }
        let query = format!("INSERT INTO {}({},{}) VALUES (?,?)", table_name, link1_name, link2_name);
        let mut smt = connection.prepare(query.as_str()).unwrap();
        //for item in ordered_set.iter() {
        //    smt.execute([book_id.as_bytes(), item.as_str().as_bytes()]);
        //}
        connection.flush_prepared_statement_cache();
    }

}