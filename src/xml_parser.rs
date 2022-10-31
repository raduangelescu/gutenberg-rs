use fast_xml::events::Event;
use fast_xml::Reader;
use indexmap::IndexSet;
use std::borrow::Borrow;
use std::fs;
use std::path::PathBuf;
use std::str;

use crate::fst_parser_node::FSTParserNode;
use crate::fst_parser_or_node::FSTParserOrNode;
use crate::fst_parser_file_node::FSTParserFileNode;
use crate::fst_parser_type::ParseType;
use crate::fst_parser::FSTParser;
use crate::book::Book;
use crate::fst_parser::ParseResult;
use crate::fst_parser::ParseItemResult;

fn parse_rdf(path: &PathBuf, field_parsers: &mut Vec<Box<dyn FSTParser>>, out:&mut ParseResult) -> usize {
    println!("Doing: {}", path.display());

    let mut reader = Reader::from_file(path).unwrap();
    let mut buf = Vec::new();
    let mut book_id:usize = 0;
   
    loop {
        reader.trim_text(true);

        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let current_node_name = str::from_utf8(e.name()).unwrap();
                if current_node_name.eq("rdf::RDF") {
                    continue;
                }
                
                if current_node_name.eq("pgterms:ebook") {
                    for attr in e.attributes() {
                        let attr_val = attr.unwrap();
                        if attr_val.key.eq(b"rdf:about") {
                            let str_book_id = str::from_utf8(attr_val.value.borrow()).unwrap();
                            let splits = str_book_id.split("/").collect::<Vec<&str>>();
                            assert!(splits.len() == 2);
                            book_id = splits[1].parse::<usize>().unwrap();
                        }
                    }
                    continue;
                }
                
                for check in field_parsers.iter_mut() {
                    check.start_node(current_node_name);
                    for attr in e.attributes() {
                        let a = attr.unwrap();
                        let value = str::from_utf8(a.value.borrow()).unwrap();
                        let key = str::from_utf8(a.key.borrow()).unwrap();
                        check.attribute(&key, &value, out);
                    }    
                }
            }

            Ok(Event::End(ref e)) => {
                let current_node_name = str::from_utf8(e.name()).unwrap();
                for check in field_parsers.iter_mut() {
                    check.end_node(current_node_name);
                }
            }

            Ok(Event::Text(ref e)) => {
                for check in field_parsers.iter_mut() {
                    check.text(e.unescape_and_decode(&reader).unwrap().as_str(), out);
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    /* check_single(&field_parsers[ParseType::Title as usize]);
    check_single(&field_parsers[ParseType::Publisher as usize]);
    check_single(&field_parsers[ParseType::Rights as usize]);
    check_single(&field_parsers[ParseType::DateIssued as usize]);
    check_single(&field_parsers[ParseType::Downloads as usize]);*/
    return book_id;
}

pub fn parse_xml(folder_path: &PathBuf) -> ParseResult {
    let paths = fs::read_dir(folder_path).unwrap();
    let mut parse_result : ParseResult = ParseResult {
        books : Vec::new(),
        data : Vec::new(),
    };
    let mut field_parsers = vec![
        FSTParserOrNode::build(vec![
                    vec!["dcterms:title".to_string()],
                    vec!["dcterms:alternative".to_string()]], ParseType::Title),
        FSTParserNode::build(
            vec!["dcterms:subject", "rdf:Description", "rdf:value"],
            ParseType::Subject,
        ),
        FSTParserNode::build(
            vec!["dcterms:language", "rdf:Description", "rdf:value"],
            ParseType::Language,
        ),
        FSTParserOrNode::build(vec![
            vec!["dcterms:creator".to_string(), "pgterms:agent".to_string(), "pgterms:name".to_string()],
            vec!["dcterms:creator".to_string(), "pgterms:agent".to_string(), "pgterms:agent".to_string()]
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

    for _ in  &field_parsers {
        parse_result.data.push(IndexSet::new());
    }

    for path in paths {
        let path_value = path.unwrap();
        let file_paths = fs::read_dir(path_value.path()).unwrap();
        for file_path in file_paths {
            let book_id = parse_rdf(&file_path.unwrap().path(), &mut field_parsers, &mut parse_result);
            let publisher_id = match field_parsers[ParseType::Publisher as usize].get_result() {
                    Ok(item) => item.item_links[0] as i32,
                    Err(_) => -1
            };
            
            let title_id = match field_parsers[ParseType::Title as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1
            };

            let rights_id = match field_parsers[ParseType::Rights as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1
            };

            let date_id = match field_parsers[ParseType::DateIssued as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1
            };

            let down_id = match field_parsers[ParseType::Downloads as usize].get_result() {
                Ok(item) => item.item_links[0] as i32,
                Err(_) => -1
            };

            let language_ids = field_parsers[ParseType::Language as usize].get_result()
            .unwrap_or(&ParseItemResult{item_links: Vec::new()})
            .item_links.clone();
            
            let subject_ids = field_parsers[ParseType::Subject as usize].get_result()
            .unwrap_or(&ParseItemResult{item_links: Vec::new()})
            .item_links.clone();
            
            let author_ids = field_parsers[ParseType::Author as usize].get_result()
            .unwrap_or(&ParseItemResult{item_links: Vec::new()})
            .item_links.clone();
            
            let bookshelf_ids = field_parsers[ParseType::Bookshelf as usize].get_result()
            .unwrap_or(&ParseItemResult{item_links: Vec::new()})
            .item_links.clone();
            
            parse_result.books.push(Book {
                publisher_id,
                title_id,
                rights_id,
                gutenberg_book_id : book_id,
                date_issued : parse_result.data[ParseType::DateIssued as usize][date_id as usize].clone(),
                num_downloads : parse_result.data[ParseType::Downloads as usize][down_id as usize].parse::<i32>().unwrap(),
                
                language_ids,
                subject_ids,
                author_ids,
                bookshelf_ids,
            });
            for parser in &mut field_parsers {
                parser.reset();
            }
        }
        for book in &parse_result.books {
            book.debug(&parse_result);
        }
    }
    parse_result
}
