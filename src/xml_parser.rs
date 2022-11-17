use fast_xml::events::Event;
use fast_xml::Reader;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Borrow;
use std::fs;
use std::path::PathBuf;
use std::str;

use crate::book::Book;
use crate::error::Error;
use crate::fst_parser::DictionaryItemContent;
use crate::fst_parser::FSTParser;
use crate::fst_parser::ParseItemResult;
use crate::fst_parser::ParseResult;
use crate::fst_parser_file_node::FSTParserFileNode;
use crate::fst_parser_node::FSTParserNode;
use crate::fst_parser_or_node::FSTParserOrNode;
use crate::fst_parser_type::ParseType;

fn parse_rdf(
    path: &PathBuf,
    field_parsers: &mut Vec<Box<dyn FSTParser>>,
    book_id: usize,
    out: &mut ParseResult,
) -> Result<usize, Error> {
    let mut reader = Reader::from_file(path)?;
    let mut buf = Vec::new();
    let mut gutenberg_book_id: usize = 0;

    loop {
        reader.trim_text(true);

        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let current_node_name = str::from_utf8(e.name())?;
                if current_node_name.eq("rdf::RDF") {
                    continue;
                }

                if current_node_name.eq("pgterms:ebook") {
                    for attr in e.attributes() {
                        let attr_val = attr?;
                        if attr_val.key.eq(b"rdf:about") {
                            let str_book_id = str::from_utf8(attr_val.value.borrow())?;
                            let splits = str_book_id.split("/").collect::<Vec<&str>>();
                            assert!(splits.len() == 2);
                            gutenberg_book_id = splits[1].parse::<usize>()?;
                        }
                    }
                    continue;
                }

                for check in field_parsers.iter_mut() {
                    check.start_node(current_node_name);
                    for attr in e.attributes() {
                        let a = attr?;
                        let value = str::from_utf8(a.value.borrow())?;
                        let key = str::from_utf8(a.key.borrow())?;
                        check.attribute(&key, &value, out, book_id as i32);
                    }
                }
            }

            Ok(Event::End(ref e)) => {
                let current_node_name = str::from_utf8(e.name())?;
                for check in field_parsers.iter_mut() {
                    check.end_node(current_node_name);
                }
            }

            Ok(Event::Text(ref e)) => {
                for check in field_parsers.iter_mut() {
                    check.text(
                        e.unescape_and_decode(&reader)?.as_str(),
                        out,
                        book_id as i32,
                    )?;
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    return Ok(gutenberg_book_id);
}

pub fn parse_xml(folder_path: &String) -> Result<ParseResult, Error> {
    let paths = fs::read_dir(folder_path)?;
    let mut parse_result: ParseResult = ParseResult {
        books: Vec::new(),
        field_dictionaries: Vec::new(),
        file_types_dictionary: IndexMap::<String, DictionaryItemContent>::default(),
        files_dictionary: IndexMap::<String, DictionaryItemContent>::default(),
    };
    let mut field_parsers = vec![
        FSTParserOrNode::build(
            vec![
                vec!["dcterms:title".to_string()],
                vec!["dcterms:alternative".to_string()],
            ],
            ParseType::Title,
        ),
        FSTParserNode::build(
            vec!["dcterms:subject", "rdf:Description", "rdf:value"],
            ParseType::Subject,
        ),
        FSTParserNode::build(
            vec!["dcterms:language", "rdf:Description", "rdf:value"],
            ParseType::Language,
        ),
        FSTParserOrNode::build(
            vec![
                vec![
                    "dcterms:creator".to_string(),
                    "pgterms:agent".to_string(),
                    "pgterms:name".to_string(),
                ],
                vec![
                    "dcterms:creator".to_string(),
                    "pgterms:agent".to_string(),
                    "pgterms:agent".to_string(),
                ],
            ],
            ParseType::Author,
        ),
        FSTParserNode::build(
            vec!["pgterms:bookshelf", "rdf:Description", "rdf:value"],
            ParseType::Bookshelf,
        ),
        FSTParserFileNode::build(
            vec![
                "dcterms:hasFormat",
                "pgterms:file",
                "dcterms:format",
                "rdf:Description",
                "rdf:value",
            ],
            "rdf:about",
            ParseType::Files,
        ),
        FSTParserNode::build(vec!["dcterms:publisher"], ParseType::Publisher),
        FSTParserNode::build(vec!["dcterms:rights"], ParseType::Rights),
        FSTParserNode::build(vec!["dcterms:issued"], ParseType::DateIssued),
        FSTParserNode::build(vec!["pgterms:downloads"], ParseType::Downloads),
    ];

    for _ in &field_parsers {
        parse_result.field_dictionaries.push(IndexMap::new());
    }

    let all_paths = paths.collect::<Vec<_>>();

    let pb = ProgressBar::new(all_paths.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})",
        )?
        .progress_chars("█  "),
    );

    pb.set_message(format!("Parsing rdf"));
    let mut idx = 0;
    for path in all_paths {
        idx = idx + 1;
        pb.set_position(idx as u64);

        let path_value = path?;
        let file_paths = fs::read_dir(path_value.path())?;
        for file_path in file_paths {
            let gutenberg_book_id = parse_rdf(
                &file_path?.path(),
                &mut field_parsers,
                idx,
                &mut parse_result,
            )?;
            let publisher_id = match field_parsers[ParseType::Publisher as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1,
            };

            let title_id = match field_parsers[ParseType::Title as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1,
            };

            let rights_id = match field_parsers[ParseType::Rights as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1,
            };

            let date_id = match field_parsers[ParseType::DateIssued as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1,
            };

            let down_id = match field_parsers[ParseType::Downloads as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1,
            };

            let language_ids = field_parsers[ParseType::Language as usize]
                .get_result()
                .unwrap_or(&ParseItemResult {
                    item_links: Vec::new(),
                })
                .item_links
                .clone();

            let subject_ids = field_parsers[ParseType::Subject as usize]
                .get_result()
                .unwrap_or(&ParseItemResult {
                    item_links: Vec::new(),
                })
                .item_links
                .clone();

            let author_ids = field_parsers[ParseType::Author as usize]
                .get_result()
                .unwrap_or(&ParseItemResult {
                    item_links: Vec::new(),
                })
                .item_links
                .clone();

            let bookshelf_ids = field_parsers[ParseType::Bookshelf as usize]
                .get_result()
                .unwrap_or(&ParseItemResult {
                    item_links: Vec::new(),
                })
                .item_links
                .clone();

            parse_result.books.push(Book {
                publisher_id,
                title_id,
                rights_id,
                gutenberg_book_id,
                date_issued: parse_result.field_dictionaries[ParseType::DateIssued as usize]
                    .get_index(date_id as usize)
                    .unwrap()
                    .0
                    .to_string(),
                num_downloads: parse_result.field_dictionaries[ParseType::Downloads as usize]
                    .get_index(down_id as usize)
                    .unwrap()
                    .0
                    .parse::<i32>()?,

                language_ids,
                subject_ids,
                author_ids,
                bookshelf_ids,
                files: field_parsers[ParseType::Files as usize]
                    .get_files()
                    .unwrap(),
            });
            for parser in &mut field_parsers {
                parser.reset();
            }
        }
    }
    pb.finish();
    Ok(parse_result)
}
