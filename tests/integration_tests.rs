use gutenberg_rs::fst_parser::ParseResult;
use gutenberg_rs::fst_parser_type::ParseType;
use gutenberg_rs::rdf_parser::parse_rdfs_from_content;
use gutenberg_rs::settings::GutenbergCacheSettings;
use gutenberg_rs::sqlite_cache::SQLiteCache;
use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;

static SAMPLE_1: &str = include_str!("../tests/documents/pg1.rdf");
static SAMPLE_2: &str = include_str!("../tests/documents/pg25.rdf");
static SAMPLE_3: &str = include_str!("../tests/documents/pg732.rdf");
static SAMPLE_4: &str = include_str!("../tests/documents/pg1000.rdf");
static SAMPLE_5: &str = include_str!("../tests/documents/pg90907.rdf");
static SAMPLE_6: &str = include_str!("../tests/documents/pg41418.rdf");

pub struct BookTitleAuthor {
    pub title: String,
    pub author: String,
}
pub struct CheckTitleAuthor {
    pub title: String,
    pub author: String,
    pub gutenberg_id: i32,
}

fn check_title_author_book_id(
    cache: &mut SQLiteCache,
    author: &str,
    title: &str,
    gutenberg_book_id: i32,
) {
    let query_string = format!("SELECT titles.name, authors.name
    FROM titles, authors, book_authors, books 
    WHERE titles.bookid = books.id AND authors.id = book_authors.authorid AND books.id = book_authors.bookid
    AND gutenbergbookid = {gutenberg_book_id}");
    let result = cache.connection.query_row("SELECT titles.name, authors.name
    FROM titles, authors, book_authors, books 
    WHERE titles.bookid = books.id AND authors.id = book_authors.authorid AND books.id = book_authors.bookid
    AND gutenbergbookid = ?1", (gutenberg_book_id,), |row| {
        Ok(BookTitleAuthor {
            title: row.get(0)?,
            author: row.get(1)?,
        })
    });
    match result {
        Ok(x) => {
            assert_eq!(x.title, title);
            assert_eq!(x.author, author);
        }
        Err(e) => {
            println!(
                "sqlite error: {}  for query {}",
                e.to_string(),
                query_string
            );
            assert!(false);
        }
    }
}
#[test]
fn test_parse_authors_titles() {
    let documents = vec![
        SAMPLE_1.to_string(),
        SAMPLE_2.to_string(),
        SAMPLE_3.to_string(),
        SAMPLE_4.to_string(),
        SAMPLE_5.to_string(),
        SAMPLE_6.to_string(),
    ];
    let check_author_titles = vec![
        CheckTitleAuthor {
            author: "Jefferson, Thomas".to_string(),
            title: "The Declaration of Independence of the United States of America".to_string(),
            gutenberg_id: 1,
        },
        CheckTitleAuthor {
            author: "United States. Central Intelligence Agency".to_string(),
            title: "The 1991 CIA World Factbook".to_string(),
            gutenberg_id: 25,
        },
        CheckTitleAuthor {
            author: "Gibbon, Edward".to_string(),
            title: "History of the Decline and Fall of the Roman Empire — Volume 2".to_string(),
            gutenberg_id: 732,
        },
        CheckTitleAuthor {
            author: "Dante Alighieri".to_string(),
            title: "La Divina Commedia di Dante: Complete".to_string(),
            gutenberg_id: 1000,
        },
        CheckTitleAuthor {
            author: "".to_string(),
            title: "".to_string(),
            gutenberg_id: 90907,
        },
        CheckTitleAuthor {
            author: "Whyte-Melville, G. J. (George John)".to_string(),
            title: "Contraband; Or, A Losing Hazard".to_string(),
            gutenberg_id: 41418,
        },
    ];

    let parsing_results: ParseResult = parse_rdfs_from_content(&documents, false).unwrap();
    assert_eq!(parsing_results.books.len(), documents.len());
    assert_eq!(parsing_results.field_dictionaries.len(), 10);
    assert_eq!(parsing_results.files_dictionary.len(), 75);
    assert_eq!(parsing_results.file_types_dictionary.len(), 12);
    // build sqlite
    let mut settings = GutenbergCacheSettings::default();
    settings.db_in_memory = true;

    let cache = SQLiteCache::create_cache(&parsing_results, &settings, true, false);

    let authors = &parsing_results.field_dictionaries[ParseType::Author as usize];
    let titles = &parsing_results.field_dictionaries[ParseType::Title as usize];

    assert_ne!(authors.len(), 0);
    assert_eq!(parsing_results.books[0].author_ids.len(), 1);
    assert_eq!(parsing_results.books[1].author_ids.len(), 1);
    assert_eq!(parsing_results.books[2].author_ids.len(), 1);
    assert_eq!(parsing_results.books[3].author_ids.len(), 1);
    assert_eq!(parsing_results.books[4].author_ids.len(), 0); // this one does not have an author
    assert_eq!(parsing_results.books[5].author_ids.len(), 1);
    match cache {
        Ok(mut x) => {
            for i in 0..6 {
                if i == 4 {
                    continue;
                }
                match authors.get_index(parsing_results.books[i].author_ids[0]) {
                    Some(x) => assert_eq!(x.0, &check_author_titles[i].author),
                    None => assert!(false),
                }
                match titles.get_index(parsing_results.books[i].title_id as usize) {
                    Some(x) => assert_eq!(x.0, &check_author_titles[i].title),
                    None => assert!(false),
                }
                check_title_author_book_id(
                    &mut x,
                    check_author_titles[i].author.as_str(),
                    check_author_titles[i].title.as_str(),
                    check_author_titles[i].gutenberg_id,
                );
            }
        }
        Err(x) => {
            println!("error {}", x.to_string());
            assert!(false);
        }
    }
}

fn compare_query_results(x: &mut SQLiteCache, json: &Value, expected: Vec<i32>) {
    let filtered_by_lang = x.query(json);
    match filtered_by_lang {
        Ok(x) => {
            let set1: HashSet<i32> = x.iter().copied().collect();
            let set2: HashSet<i32> = expected.into_iter().collect();
            assert_eq!(set1, set2);
        }
        Err(x) => {
            println!("error {}", x.to_string());
            assert!(false);
        }
    }
}
#[test]
fn test_query() {
    let documents = vec![
        SAMPLE_1.to_string(),
        SAMPLE_2.to_string(),
        SAMPLE_3.to_string(),
        SAMPLE_4.to_string(),
        SAMPLE_5.to_string(),
        SAMPLE_6.to_string(),
    ];
    let mut settings = GutenbergCacheSettings::default();
    settings.db_in_memory = true;
    let parsing_results: ParseResult = parse_rdfs_from_content(&documents, false).unwrap();
    let cache = SQLiteCache::create_cache(&parsing_results, &settings, true, false);
    match cache {
        Ok(mut x) => {
            compare_query_results(
                &mut x,
                &json!({"language": "\"en\""}),
                vec![41418, 25, 732, 90907, 1],
            );
            compare_query_results(
                &mut x,
                &json!({"language": "\"en\"", "author": "\"Jefferson, Thomas\""}),
                vec![1],
            );
            compare_query_results(
                &mut x,
                &json!({"language": "\"en\"", "rights": "\"Public domain in the USA.\""}),
                vec![90907, 732, 41418, 1, 25],
            );
            compare_query_results(
                &mut x,
                &json!({"language": "\"en\"", "downloadlinkstype": "\"image/jpeg\""}),
                vec![732, 1, 41418, 25],
            );
            compare_query_results(
                &mut x,
                &json!({"language": "\"en\"", "bookshelve": "\"IT Poesia\""}),
                vec![],
            );
            compare_query_results(
                &mut x,
                &json!({"language": "\"it\"", "bookshelve": "\"IT Poesia\""}),
                vec![1000],
            );
        }
        Err(x) => {
            println!("error {}", x.to_string());
            assert!(false);
        }
    }
}
