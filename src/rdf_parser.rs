use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Borrow;
use std::str;
use std::io::BufReader;
use std::fs;
use walkdir::WalkDir;

use quick_xml::reader::Reader;
use quick_xml::events::Event;

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

pub trait XmlReader {
    fn trim(&mut self, val: bool) -> &mut Self;
    fn read<'b>(&mut self, buf: &'b mut Vec<u8>) -> quick_xml::Result<Event<'b>>;
    fn pos(&self) -> usize;
}

impl XmlReader for Reader<BufReader<std::fs::File>> {
    fn trim(&mut self, val: bool) -> &mut Self { self.trim_text(val)}
    fn read<'b>(&mut self, buf: &'b mut Vec<u8>) -> quick_xml::Result<Event<'b>> {self.read_event_into(buf)}
    fn pos(&self) -> usize {self.buffer_position()}
}

impl XmlReader for Reader<&[u8]> {
    fn trim(&mut self, val: bool) -> &mut Self { self.trim_text(val)}
    fn read<'b>(&mut self, buf: &'b mut Vec<u8>) -> quick_xml::Result<Event<'b>> {self.read_event_into(buf)}
    fn pos(&self) -> usize {self.buffer_position()}
}

pub fn parse_rdf_from_reader<R: XmlReader>(reader: &mut R,
    field_parsers: &mut Vec<Box<dyn FSTParser>>,
    book_id: usize,
    out: &mut ParseResult
)  -> Result<usize, Error>  {
    let mut gutenberg_book_id: usize = 0;
    let mut buf = Vec::with_capacity(1024);
    loop {
        reader.trim(true);

        match reader.read(&mut buf) {
            Ok(Event::Start(e)) => {
                let current_node_name = str::from_utf8(e.name().0)?;
                if current_node_name.eq("rdf::RDF") {
                    continue;
                }

                if current_node_name.eq("pgterms:ebook") {
                    for attr in e.attributes() {
                        let attr_val = attr?;
                        if attr_val.key.0.eq(b"rdf:about") {
                            let str_book_id = str::from_utf8(attr_val.value.borrow())?;
                            let splits = str_book_id.split("/").collect::<Vec<&str>>();
                            assert!(splits.len() == 2);
                            match splits[1].parse::<usize>() {
                                Ok(book_id) => { 
                                    gutenberg_book_id = book_id;
                                },
                                Err(e) => {
                                    return Err(Error::InvalidRdf(
                                    format!(
                                        "parseIntError:{} , cannot parse bookid for {}",
                                        e.to_string(),
                                        book_id
                                    )
                                    .to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    continue;
                }

                for check in field_parsers.iter_mut() {
                    check.start_node(current_node_name);
                    for attr in e.attributes() {
                        let a = attr?;
                        let value = str::from_utf8(a.value.borrow())?;
                        let key = str::from_utf8(a.key.0.borrow())?;
                        check.attribute(&key, &value, out, book_id as i32)?;
                    }
                }
            }

            Ok(Event::End(ref e)) => {
                let current_node_name = str::from_utf8(e.name().0)?;
                for check in field_parsers.iter_mut() {
                    check.end_node(current_node_name);
                }
            }

            Ok(Event::Text(ref e)) => {
                for check in field_parsers.iter_mut() {
                    check.text(
                        e.unescape()?.into_owned().as_str(),
                        out,
                        book_id as i32,
                    )?;
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.pos(), e),
            _ => (),
        }
    }
    return Ok(gutenberg_book_id);
}

fn setup_fst() -> (ParseResult, Vec<Box<dyn FSTParser>>) {
    let mut parse_result: ParseResult = ParseResult {
        books: Vec::with_capacity(1024),
        field_dictionaries: Vec::with_capacity(1024),
        file_types_dictionary: IndexMap::<String, DictionaryItemContent>::with_capacity(1024),
        files_dictionary: IndexMap::<String, DictionaryItemContent>::with_capacity(1024),
    };
    let field_parsers = vec![
        FSTParserOrNode::build(
            vec![
                vec!["dcterms:title"],
                vec!["dcterms:alternative"],
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
                    "dcterms:creator",
                    "pgterms:agent",
                    "pgterms:name",
                ],
                vec![
                    "dcterms:creator",
                    "pgterms:agent",
                    "pgterms:agent",
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

    (parse_result, field_parsers)
}

fn get_files_from_directory(folder_path: &String) -> Result<Vec<String>, Error> {
    let paths = WalkDir::new(folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| { match e.metadata() {
            Ok(e) => e.is_file(),
            _ => false,
        }})
        .map(|e| { e.path().display().to_string()})
        .collect::<Vec<String>>();
    Ok(paths)
}

pub fn parse_rdfs_from_folder(folder: &String, display_progress_bar: bool) -> Result<ParseResult, Error> {
    let paths = get_files_from_directory(folder)?;
    parse_rdfs(&paths, false, display_progress_bar)
}

pub fn parse_rdfs_from_content(rdfs_content: &Vec<String>, display_progress_bar: bool) -> Result<ParseResult, Error> {
    parse_rdfs(&rdfs_content, true, display_progress_bar)
}

fn parse_rdfs(param: &Vec<String>, is_content: bool, display_progress_bar: bool) -> Result<ParseResult, Error> {
    
    let ( mut parse_result, mut field_parsers) = setup_fst();
    
    let mut pb: Option<ProgressBar> = None;
    if display_progress_bar {
        let pb_new = ProgressBar::new(param.len() as u64);
        pb_new.set_style(
            ProgressStyle::with_template(
                "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] ({eta})",
            )?
            .progress_chars("█  "),
        );
        pb_new.set_message(format!("Parsing rdf"));
        pb = Some(pb_new);
    }
    let mut idx = 0;
    for file_path in param {
        idx = idx + 1;
        
        match pb {
            Some(ref p) => p.set_position(idx as u64),
            _ => {}
        }

        let gutenberg_book_id;
        let mut reader;
        let data;
        if is_content {
            reader = Reader::from_str(file_path); 
        }
        else {
            data = fs::read_to_string(file_path)?;
            reader = Reader::from_str(data.as_str());
        }
        
        gutenberg_book_id = parse_rdf_from_reader(
            &mut reader,
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
        let mut date_issued = "".to_string();
        if let Some(dict_value) = parse_result.field_dictionaries
            [ParseType::DateIssued as usize]
            .get_index(date_id as usize)
        {
            date_issued = dict_value.0.to_string();
        }

        let mut num_downloads = 0;
        if let Some(dict_value) = parse_result.field_dictionaries
            [ParseType::Downloads as usize]
            .get_index(down_id as usize)
        {
            match dict_value.0.parse::<i32>() {
                Ok(val) => { 
                    num_downloads = val;
                },
                Err(e) => {
                    return Err(Error::InvalidRdf(format!("bad num downloads parse for book {}, {}, {}",  gutenberg_book_id, e.to_string(), dict_value.0).to_string()));
                }
            }
        }

        parse_result.books.push(Book {
            publisher_id,
            title_id,
            rights_id,
            gutenberg_book_id,
            date_issued,
            num_downloads,
            language_ids,
            subject_ids,
            author_ids,
            bookshelf_ids,
            files: field_parsers[ParseType::Files as usize].get_files()?,
        });
        for parser in &mut field_parsers {
            parser.reset();
        }
    }
    match pb {
        Some(ref p) => p.finish(),
        _ => {}
    }

    Ok(parse_result)
}
