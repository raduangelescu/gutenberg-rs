
use sqlite_cache::SQLiteCache;
use crate::error::Error;

use settings::GutenbergCacheSettings;
use utils::{download_file, decompress_tar_bz};

mod book;
mod fst_parser;
mod fst_parser_file_node;
mod fst_parser_node;
mod fst_parser_or_node;
mod fst_parser_type;
mod xml_parser;
mod utils;

pub mod settings;
pub mod textget;
pub mod sqlite_cache;
pub mod error;
pub mod db_cache;

pub async fn setup_sqlite(settings: &GutenbergCacheSettings) -> Result<(), Error> {

    download_file(
        &settings.cache_rdf_download_link,
        &settings.cache_rdf_archive_name,
    ).await?;

    decompress_tar_bz(&settings.cache_rdf_archive_name)?;

    let parse_result = xml_parser::parse_xml(&settings.cache_rdf_unpack_directory)?;
    
    SQLiteCache::create_cache(&parse_result, settings, None)?;
    Ok(())
}