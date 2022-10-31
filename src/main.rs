use bzip2::read::BzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use tar::Archive;

mod xml_parser;
mod fst_parser;
mod fst_parser_node;
mod fst_parser_or_node;
mod fst_parser_file_node;
mod fst_parser_type;
mod db_cache;
mod sqlite_cache;
mod book;


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

fn decompress_bz(path: &str) -> Result<(u64, String), std::io::Error> {
    let bz_file = File::open(path)?;
    let bz_size = bz_file.metadata().unwrap().len();
    let new_filename = &path[..path.len() - 3];

    let pb = ProgressBar::new(bz_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    .unwrap()
    .progress_chars("█  "));

    pb.set_message(format!("Decompressing {} to {}", path, new_filename));

    let mut decoder = BzDecoder::new(bz_file);
    let big_data_size = 1024 * 1024;
    let mut total_archive_size = 0 as u64;
    let mut output_file = File::create(new_filename).unwrap();

    loop {
        let mut read_buffer = vec![0; big_data_size];
        let data_len = decoder.read(&mut read_buffer).unwrap() as u64;
        total_archive_size += data_len;
        if big_data_size > data_len as usize {
            read_buffer.resize(data_len as usize, 0);
        }
        output_file.write(&read_buffer)?;
        pb.set_position(decoder.total_in());
        if decoder.total_in() == bz_size {
            decoder.flush();
            output_file.flush();
            break;
        }
    }
    pb.finish();
    Ok((total_archive_size, new_filename.to_string()))
}

fn decompress_tar(path: &str, initial_size: u64) -> Result<(), std::io::Error> {
    let tar = File::open(path)?;
    let mut archive = Archive::new(tar);

    let pb = ProgressBar::new(initial_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    .unwrap()
    .progress_chars("█  "));

    pb.set_message(format!("Unpacking to folder"));

    archive.entries().unwrap().for_each(|entry| {
        let mut entry_value = entry.unwrap();
        entry_value.unpack_in(".").unwrap();
        pb.set_position(entry_value.raw_header_position() as u64);
    });
    pb.finish();
    Ok(())
}

#[tokio::main]
async fn main() {
    let filename = "gutenberg.tar.bz2";

    //download_file(&Client::new(), "https://www.gutenberg.org/cache/epub/feeds/rdf-files.tar.bz2", "gutenberg.tar.bz2").await.unwrap();
    //let (total_archive_size, bz_filename) = decompress_bz(filename).unwrap();
    //decompress_tar(bz_filename.as_str(), total_archive_size).unwrap();
    let folder = Path::new("cache").join("epub");
    xml_parser::parse_xml(&folder);
}
