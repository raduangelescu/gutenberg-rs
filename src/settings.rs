use std::path::Path;
use crate::error::Error;
use num_traits::ToPrimitive;
use serde_json::Value;

pub struct GutenbergCacheSettings {
    pub cache_rdf_download_link: String,
    pub cache_filename: String,
    pub cache_rdf_unpack_directory: String,
    pub cache_rdf_archive_name: String,
    pub download_num_divs: i32,
    pub text_files_cache_folder: String,
    pub mongo_db_connection_server: String,
}

impl Default for GutenbergCacheSettings {
    fn default() -> GutenbergCacheSettings {
        GutenbergCacheSettings {
            text_files_cache_folder: "text_cache".to_string(),
            cache_rdf_download_link: "https://www.gutenberg.org/cache/epub/feeds/rdf-files.tar.bz2".to_string(),
            cache_filename: "gutenbergindex.db".to_string(),
            cache_rdf_unpack_directory: Path::new("cache").join("epub").as_path().display().to_string(),
            cache_rdf_archive_name: "rdf-files.tar.bz2".to_string(),
            download_num_divs: 20,
            mongo_db_connection_server: "mongodb://localhost:27017".to_string(),
        }
    }
}

impl GutenbergCacheSettings {
    pub fn from(json: &Value) -> Result<GutenbergCacheSettings, Error> {
        let mut settings = GutenbergCacheSettings::default();
        if let Some(field) = json.get("CacheFilename") {
            if let Some(v) = field.as_str() {
                settings.cache_filename = v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("CacheFilename".to_string()));
            }
        }
        if let Some(field) = json.get("CacheUnpackDir") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_unpack_directory = v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("CacheUnpackDir".to_string()));
            }
        }
        if let Some(field) = json.get("CacheArchiveName") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_archive_name =  v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("CacheArchiveName".to_string()));
            }
        }
        if let Some(field) = json.get("ProgressBarMaxLength") {
            if let Some(v) = field.as_i64() {
                if let Some(vi32) = v.to_i32() {
                    settings.download_num_divs = vi32;
                }
                else {
                    return Err(Error::InvalidSettingsField("ProgressBarMaxLength".to_string()));
                }
            }
            else {
                return Err(Error::InvalidSettingsField("ProgressBarMaxLength".to_string()));
            }
        }
        if let Some(field) = json.get("CacheRDFDownloadLink") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_download_link = v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("CacheRDFDownloadLink".to_string()));
            }
        }
        if let Some(field) = json.get("TextFilesCacheFolder") {
            if let Some(v) = field.as_str() {
                settings.text_files_cache_folder = v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("TextFilesCacheFolder".to_string()));
            }
        }
        if let Some(field) = json.get("MongoDBCacheServer") {
            if let Some(v) = field.as_str() {
                settings.mongo_db_connection_server = v.to_string();
            }
            else {
                return Err(Error::InvalidSettingsField("MongoDBCacheServer".to_string()));
            }
        }
        Ok(settings)
    }
}