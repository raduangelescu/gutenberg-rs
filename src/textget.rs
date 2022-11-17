//
// MARKERS ARE FROM https://github.com/c-w/Gutenberg/blob/master/gutenberg/_domain_model/text.py
use std::path::{Path};
use std::fs::create_dir_all;
use std::fs;
use std::error::Error;
use url::{Url, Position};
use reqwest::Client;

use crate::settings::GutenbergCacheSettings;

const TEXT_START_MARKERS: &[&str] = &[
    "*END*THE SMALL PRINT",
    "*** START OF THE PROJECT GUTENBERG",
    "*** START OF THIS PROJECT GUTENBERG",
    "This etext was prepared by",
    "E-text prepared by",
    "Produced by",
    "Distributed Proofreading Team",
    "Proofreading Team at http://www.pgdp.net",
    "http://gallica.bnf.fr)",
    "      http://archive.org/details/",
    "http://www.pgdp.net",
    "by The Internet Archive)",
    "by The Internet Archive/Canadian Libraries",
    "by The Internet Archive/American Libraries",
    "public domain material from the Internet Archive",
    "Internet Archive)",
    "Internet Archive/Canadian Libraries",
    "Internet Archive/American Libraries",
    "material from the Google Print project",
    "*END THE SMALL PRINT",
    "***START OF THE PROJECT GUTENBERG",
    "This etext was produced by",
    "*** START OF THE COPYRIGHTED",
    "The Project Gutenberg",
    "http://gutenberg.spiegel.de/ erreichbar.",
    "Project Runeberg publishes",
    "Beginning of this Project Gutenberg",
    "Project Gutenberg Online Distributed",
    "Gutenberg Online Distributed",
    "the Project Gutenberg Online Distributed",
    "Project Gutenberg TEI",
    "This eBook was prepared by",
    "http://gutenberg2000.de erreichbar.",
    "This Etext was prepared by",
    "This Project Gutenberg Etext was prepared by",
    "Gutenberg Distributed Proofreaders",
    "Project Gutenberg Distributed Proofreaders",
    "the Project Gutenberg Online Distributed Proofreading Team",
    "**The Project Gutenberg",
    "*SMALL PRINT!",
    "More information about this book is at the top of this file.",
    "tells you about restrictions in how the file may be used.",
    "l'authorization à les utilizer pour preparer ce texte.",
    "of the etext through OCR.",
    "*****These eBooks Were Prepared By Thousands of Volunteers!*****",
    "We need your donations more than ever!",
    " *** START OF THIS PROJECT GUTENBERG",
    "****     SMALL PRINT!",
    "[\"Small Print\" V.",
    "      (http://www.ibiblio.org/gutenberg/",
    "and the Project Gutenberg Online Distributed Proofreading Team",
    "Mary Meehan, and the Project Gutenberg Online Distributed Proofreading",
    "                this Project Gutenberg edition.",
];

const TEXT_END_MARKERS: &[&str] = &[
    "*** END OF THE PROJECT GUTENBERG",
    "*** END OF THIS PROJECT GUTENBERG",
    "***END OF THE PROJECT GUTENBERG",
    "End of the Project Gutenberg",
    "End of The Project Gutenberg",
    "Ende dieses Project Gutenberg",
    "by Project Gutenberg",
    "End of Project Gutenberg",
    "End of this Project Gutenberg",
    "Ende dieses Projekt Gutenberg",
    "        ***END OF THE PROJECT GUTENBERG",
    "*** END OF THE COPYRIGHTED",
    "End of this is COPYRIGHTED",
    "Ende dieses Etextes ",
    "Ende dieses Project Gutenber",
    "Ende diese Project Gutenberg",
    "**This is a COPYRIGHTED Project Gutenberg Etext, Details Above**",
    "Fin de Project Gutenberg",
    "The Project Gutenberg Etext of ",
    "Ce document fut presente en lecture",
    "Ce document fut présenté en lecture",
    "More information about this book is at the top of this file.",
    "We need your donations more than ever!",
    "END OF PROJECT GUTENBERG",
    " End of the Project Gutenberg",
    " *** END OF THIS PROJECT GUTENBERG",
];

const LEGALESE_START_MARKERS: &[&str] = &["<<THIS ELECTRONIC VERSION OF",];
const LEGALESE_END_MARKERS: &[&str] = &["SERVICE THAT CHARGES FOR DOWNLOAD",];

async fn _download_content(link: &String) -> Result<String, Box<dyn Error>>  {
    let client = &Client::new();
    let request = client.get(link);
    let content = request.send().await;
    let string = content.unwrap().text().await.unwrap_or("none".to_string());
    Ok(string)
}

pub async fn get_text_by_id(settings: &GutenbergCacheSettings, link: &String) -> Result<String, std::io::Error> {
    println!("link {}", link);
    let the_url = &Url::parse(link).unwrap()[Position::AfterHost .. Position::AfterPath];
    let file_link = the_url.split_terminator("/").last().unwrap_or("");

    let file_cache_location = Path::new(settings.text_files_cache_folder.as_str()).join(file_link);
    let folder_path = Path::new(settings.text_files_cache_folder.as_str());

    if !file_cache_location.exists() {
        if !folder_path.exists() {
            create_dir_all(folder_path)?;
        }
        let content = _download_content(link).await.unwrap_or("none".to_string());
        println!("writing file {}", file_cache_location.display());
        fs::write(file_cache_location, &content).expect("Unable to write file");
        return Ok(content);
    }
    else {
        fs::read_to_string(file_cache_location)
    }
}

fn line_starts_with_any(line: &str, tokens: &[&str]) -> bool {
    for token in tokens {
        if line.starts_with(token) {
            return true;
        }
    }
    return false;
}

pub fn strip_headers(text: String) -> String {
#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

    let lines = text.split(LINE_ENDING);
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;
    let mut footer_found = false;
    let mut ignore_section = false;

    for line in lines {
        let mut reset = false;

        if i <= 600 {
            // Check if the header ends here
            if line_starts_with_any(line, TEXT_START_MARKERS) {
                reset = true
            }
            // If it's the end of the header, delete the output produced so far.
            // May be done several times, if multiple lines occur indicating the
            // end of the header
            if reset {
                out = Vec::new();
                continue
            }
        }
        if i >= 100 {
            // Check if the footer begins here
            if line_starts_with_any(line, TEXT_END_MARKERS) {
                footer_found = true;
            }
            // If it's the beginning of the footer, stop output
            if footer_found {
                break
            }
        }
        if line_starts_with_any(line, LEGALESE_START_MARKERS) {
            ignore_section = true;
            continue;
        }
        else if line_starts_with_any(line, LEGALESE_END_MARKERS) {
            ignore_section = false;
            continue;
        }
        if !ignore_section {
            let stripline = line.trim_end_matches(LINE_ENDING);
            out.push(stripline);
            i = i + 1;
        }
    }

    return out.join(LINE_ENDING);
}
    