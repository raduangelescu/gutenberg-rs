use fast_xml::events::Event;
use fast_xml::Reader;
use std::borrow::Borrow;
use std::fs;
use std::path::PathBuf;
use std::str;

use crate::fst_parser_node::FSTParserNode;
use crate::fst_parser_type::ParseType;
use crate::fst_parser::FSTParser;

fn parse_rdf(path: &PathBuf, all_checks: &mut Vec<FSTParserNode>) -> i32 {
    println!("Doing: {}", path.display());

    let mut reader = Reader::from_file(path).unwrap();
    let mut buf = Vec::new();
    let mut book_id = -1;
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
                            book_id = splits[1].parse::<i32>().unwrap();
                        }
                    }
                    assert!(book_id != -1);
                    continue;
                }

                //e.unescape_and_decode(&reader).unwrap().as_str().clone_into(&mut current_node_name);
                for check in all_checks.iter_mut() {
                    check.start_node(current_node_name, e.attributes());
                }
            }
            Ok(Event::End(ref e)) => {
                let current_node_name = str::from_utf8(e.name()).unwrap();
                for check in all_checks.iter_mut() {
                    check.end_node(current_node_name);
                }
            }
            Ok(Event::Text(ref e)) => {
                for check in all_checks.iter_mut() {
                    check.text(e.unescape_and_decode(&reader).unwrap().as_str());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    return book_id;
}

pub fn parse_xml(folder_path: &PathBuf) {
    let paths = fs::read_dir(folder_path).unwrap();
    let mut all_checks = vec![
        FSTParserNode::build(vec!["dcterms:title"], ParseType::Title, ""),
        FSTParserNode::build(vec!["dcterms:alternative"], ParseType::TitleAlternative, ""),
        FSTParserNode::build(
            vec!["dcterms:subject", "rdf:Description", "rdf:value"],
            ParseType::Subject,
            "",
        ),
        FSTParserNode::build(
            vec!["dcterms:language", "rdf:Description", "rdf:value"],
            ParseType::Language,
            "",
        ),
        FSTParserNode::build(
            vec!["dcterms:creator", "pgterms:agent", "pgterms:name"],
            ParseType::Author,
            "",
        ),
        FSTParserNode::build(
            vec!["dcterms:creator", "pgterms:agent", "pgterms:agent"],
            ParseType::AuthorAlternative,
            "",
        ),
        FSTParserNode::build(
            vec!["pgterms:bookshelf", "rdf:Description", "rdf:value"],
            ParseType::Bookshelf,
            "",
        ),
        FSTParserNode::build(
            vec!["dcterms:hasFormat", "pgterms:file"],
            ParseType::FilesLinks,
            "rdf:about",
        ),
        FSTParserNode::build(
            vec![
                "dcterms:hasFormat",
                "pgterms:file",
                "dcterms:format",
                "rdf:Description",
                "rdf:value",
            ],
            ParseType::FilesType,
            "",
        ),
        FSTParserNode::build(vec!["dcterms:publisher"], ParseType::Publisher, ""),
        FSTParserNode::build(vec!["dcterms:rights"], ParseType::Rights, ""),
        FSTParserNode::build(vec!["dcterms:issued"], ParseType::DateIssued, ""),
        FSTParserNode::build(vec!["pgterms:downloads"], ParseType::Downloads, ""),
    ];
    for path in paths {
        let path_value = path.unwrap();
        let file_paths = fs::read_dir(path_value.path()).unwrap();
        for file_path in file_paths {
            let book_id = parse_rdf(&file_path.unwrap().path(), &mut all_checks);
            println!("BookId: {}", book_id);
            for check in all_checks.iter_mut() {
                if check.has_results() {
                    println!(
                        "{}: {}",
                        check.get_parse_type().to_string(),
                        check.results.join("|")
                    );
                }
                check.reset();
            }
            println!("-------------------------------------------------------------")
        }
    }
}
