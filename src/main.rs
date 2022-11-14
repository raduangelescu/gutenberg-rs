use bzip2::read::BzDecoder;
use db_cache::DBCache;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sqlite_cache::SQLiteCache;
use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use tar::Archive;
use serde_json::json;
use textget::{get_text_by_id, strip_headers};
use settings::GutenbergCacheSettings;

mod book;
mod db_cache;
mod error;
mod fst_parser;
mod fst_parser_file_node;
mod fst_parser_node;
mod fst_parser_or_node;
mod fst_parser_type;
mod sqlite_cache;
mod xml_parser;
mod textget;
mod settings;

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let mut file;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    if std::path::Path::new(path).exists() {
        println!("File exists. Resuming.");
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .unwrap();

        let file_size = std::fs::metadata(path).unwrap().len();
        file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
        downloaded = file_size;
    } else {
        println!("Fresh file..");
        file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    }

    println!("Commencing transfer");

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .unwrap()
    .progress_chars("█  "));

    pb.set_message(format!("Downloading {} from {}", path, url));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish();
    return Ok(());
}

fn decompress_bz(path: &str) -> Result<(u64, String), Box<dyn Error>> {
    let bz_file = File::open(path)?;
    let bz_size = bz_file.metadata().unwrap().len();
    let new_filename = &path[..path.len() - 3];

    let pb = ProgressBar::new(bz_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    ?
    .progress_chars("█  "));

    pb.set_message(format!("Decompressing {} to {}", path, new_filename));

    let mut decoder = BzDecoder::new(bz_file);
    let big_data_size = 1024 * 1024;
    let mut total_archive_size = 0 as u64;
    let mut output_file = File::create(new_filename)?;

    loop {
        let mut read_buffer = vec![0; big_data_size];
        let data_len = decoder.read(&mut read_buffer)? as u64;
        total_archive_size += data_len;
        if big_data_size > data_len as usize {
            read_buffer.resize(data_len as usize, 0);
        }
        output_file.write(&read_buffer)?;
        pb.set_position(decoder.total_in());
        if decoder.total_in() == bz_size {
            decoder.flush()?;
            output_file.flush()?;
            break;
        }
    }
    pb.finish();
    Ok((total_archive_size, new_filename.to_string()))
}

fn decompress_tar(path: &str, initial_size: u64) -> Result<(), Box<dyn Error>> {
    let tar = File::open(path)?;
    let mut archive = Archive::new(tar);

    let pb = ProgressBar::new(initial_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    ?.progress_chars("█  "));

    pb.set_message(format!("Unpacking to folder"));

    archive.entries()?.for_each(|entry| {
        let mut entry_value = entry.unwrap();
        entry_value.unpack_in(".").err();
        pb.set_position(entry_value.raw_header_position() as u64);
    });
    pb.finish();
    Ok(())
}

pub async fn exec() -> Result<(), Box<dyn Error>> {
    /*let filename = "gutenberg.tar.bz2";

    download_file(
        &Client::new(),
        "https://www.gutenberg.org/cache/epub/feeds/rdf-files.tar.bz2",
        "gutenberg.tar.bz2",
    )
    .await?;
    let (total_archive_size, bz_filename) = decompress_bz(filename)?;
    decompress_tar(bz_filename.as_str(), total_archive_size)?;
    */
    //let folder = Path::new("cache").join("epub");
    //let parse_result = xml_parser::parse_xml(&folder)?;
    
    let mut cache_settings = sqlite_cache::SQLiteCacheSettings::default();
    
    //let mut cache = SQLiteCache::create_cache(&parse_result, cache_settings)?;
    
    let mut cache = SQLiteCache::get_cache(cache_settings)?;
    let res = cache.query(&json!({
        "languages": "\"en\"",
    }))?;
    let settings = GutenbergCacheSettings::default();
    for (idx, r) in res.iter().enumerate() {
        println!("getting text for gutenberg idx: {}", r);
        let links = cache.get_download_links(vec![*r])?;
        for link in links {
            let res = get_text_by_id(&settings, &link, *r).await.unwrap();
        }
        if idx > 20 {
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    match exec().await {
        Ok(_) => (),
        Err(ex) => {
            println!("ERROR = {}", ex);
            return;
        }
    }
}
