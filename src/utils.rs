use crate::error::Error;
use bzip2::read::BzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::fs::File;
use std::io::{Read, Seek, Write};
use tar::Archive;

pub async fn download_file(url: &str, path: &str) -> Result<(), Error> {
    let client = &Client::new();
    let res = client.get(url).send().await.or(Err(Error::InvalidRequest(
        format!("Failed to GET from '{}'", &url).to_string(),
    )))?;

    let total_size = res.content_length().ok_or(Error::InvalidRequest(
        format!("Bad file from '{}'", &url).to_string(),
    ))?;

    let mut file;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    if std::path::Path::new(path).exists() {
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)?;

        let file_size = std::fs::metadata(path)?.len();
        file.seek(std::io::SeekFrom::Start(file_size))?;
        downloaded = file_size;
    } else {
        file = File::create(path).or(Err(Error::InvalidRequest(format!(
            "Failed to create file '{}'",
            path
        ))))?;
    }

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
    .progress_chars("█  "));

    pb.set_message(format!("Downloading {} from {}", path, url));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(Error::InvalidRequest(format!(
            "Error while downloading file"
        ))))?;
        file.write(&chunk).or(Err(Error::InvalidRequest(format!(
            "Error while writing to file"
        ))))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish();
    return Ok(());
}

pub fn decompress_tar_bz(path: &str) -> Result<(), Error> {
    let (total_archive_size, bz_filename) = decompress_bz(path)?;
    decompress_tar(bz_filename.as_str(), total_archive_size)?;
    Ok(())
}

pub fn decompress_bz(path: &str) -> Result<(u64, String), Error> {
    let bz_file = File::open(path)?;
    let bz_size = bz_file.metadata()?.len();
    let new_filename = &path[..path.len() - 3];

    let pb = ProgressBar::new(bz_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    ?.progress_chars("█  "));

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

pub fn decompress_tar(path: &str, initial_size: u64) -> Result<(), Error> {
    let tar = File::open(path)?;
    let mut archive = Archive::new(tar);

    let pb = ProgressBar::new(initial_size);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({eta})")
    ?.progress_chars("█  "));

    pb.set_message(format!("Unpacking to folder"));
    for entry in archive.entries()? 
    {
        let mut entry_value = entry?;
        entry_value.unpack_in(".")?;
        pb.set_position(entry_value.raw_header_position() as u64);
    }

    pb.finish();
    Ok(())
}
