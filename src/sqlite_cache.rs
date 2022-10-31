
use std::ptr::null;

use crate::db_cache::DBCache;
use crate::fst_parser_type::ParseType;
use sqlite::{Connection, Cursor};
use tokio::fs::File;

struct SQLiteCache {
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
    fn create_cache(&mut self, parse_results: &Vec<String>) { 
        //let mut connection = sqlite::open(self.sqlite_db_create_cache_filename).unwrap();
        //let create_query = std::fs::read_to_string(self.sqlite_db_create_cache_filename).unwrap();
        //connection.execute(create_query).unwrap();
        
        /*for (idx,  result) in parse_results.iter().enumerate() {
            if idx == ParseType::File as usize {
                self.insert_many_fields(connection, table, ordered_set, pt.setTypes)
                self.connection.executemany( "INSERT OR IGNORE INTO downloadlinks(name,bookid,downloadtypeid) VALUES (?,?,?)"
                , map(|x| => (x[0], x[1], x[2]) parse_results.field_sets[Fields.FILES].setLinks))
            }
            else if (result.needs_book_id()) {
                self.__insert_many_field_id(self.table_map[idx], "name", "bookid", pt.set)
            }
            else {
                self.__insert_many_field(self.table_map[idx], "name", pt.set)
            }
        }
        for (idx, )*/
    }
    fn query(&self) {

    }
    fn native_query(self, query: &str) {

    }
}
type OrderedSet = std::collections::BTreeSet<String>;

impl SQLiteCache {
    fn insert_many_fields(&mut self, table: &str, ordered_set: OrderedSet) {

    }
}