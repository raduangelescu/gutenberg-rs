#![doc = include_str!("../README.md")]
use crate::error::Error;
use sqlite_cache::SQLiteCache;

use settings::GutenbergCacheSettings;
use utils::{decompress_tar_bz, download_file};

mod book;
mod fst_parser_file_node;
mod fst_parser_node;
mod fst_parser_or_node;
mod utils;

pub mod error;
pub mod rdf_parser;
pub mod settings;
pub mod sqlite_cache;
pub mod text_get;

pub mod fst_parser;
pub mod fst_parser_type;

pub async fn setup_sqlite(
    settings: &GutenbergCacheSettings,
    force_regenerate: bool,
    show_progress_bar: bool,
) -> Result<SQLiteCache, Error> {
    let archive_exists = std::path::Path::new(&settings.cache_rdf_archive_name).exists();
    if !archive_exists || force_regenerate {
        if archive_exists {
            std::fs::remove_file(&settings.cache_rdf_archive_name)?;
        }
        download_file(
            &settings.cache_rdf_download_link,
            &settings.cache_rdf_archive_name,
            show_progress_bar,
        )
        .await?;
    }

    let cache_folder_exists = std::path::Path::new(&settings.cache_rdf_unpack_directory).exists();

    if !cache_folder_exists || force_regenerate {
        if cache_folder_exists {
            std::fs::remove_file(&settings.cache_rdf_unpack_directory)?;
        }
        decompress_tar_bz(&settings.cache_rdf_archive_name, show_progress_bar)?;
    }

    match SQLiteCache::get_cache(settings) {
        Ok(cache) => Ok(cache),
        Err(_e) => {
            let parse_result =
                rdf_parser::parse_rdfs_from_folder(&settings.cache_rdf_unpack_directory, true)?;
            SQLiteCache::create_cache(&parse_result, settings, false, show_progress_bar)
        }
    }
}
