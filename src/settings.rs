use crate::error::Error;
use serde_json::Value;
use std::path::Path;

/// These are the essential settings for building your cache
pub struct GutenbergCacheSettings {
    /// This is the link used to download the rdf tar archive of rdfs from gutenberg
    pub cache_rdf_download_link: String,
    /// This is the filename of the cache db
    pub cache_filename: String,
    /// this is the directory used to unpack the rdf tar archive downloaded from gutenberg
    pub cache_rdf_unpack_directory: String,
    /// this is the archive filename in which we download
    pub cache_rdf_archive_name: String,
    /// this is the folder used to hold all the raw text data you download
    pub text_files_cache_folder: String,
    /// this will make the cache in memory (it will not save it on disk), it is used in tests
    pub db_in_memory: bool,
}

impl Default for GutenbergCacheSettings {
    fn default() -> GutenbergCacheSettings {
        GutenbergCacheSettings {
            db_in_memory: false,
            text_files_cache_folder: "text_cache".to_string(),
            cache_rdf_download_link: "https://www.gutenberg.org/cache/epub/feeds/rdf-files.tar.bz2"
                .to_string(),
            cache_filename: "gutenbergindex.db".to_string(),
            cache_rdf_unpack_directory: Path::new("cache")
                .join("epub")
                .as_path()
                .display()
                .to_string(),
            cache_rdf_archive_name: "rdf-files.tar.bz2".to_string(),
        }
    }
}

impl GutenbergCacheSettings {
    pub fn from(json: &Value) -> Result<GutenbergCacheSettings, Error> {
        let mut settings = GutenbergCacheSettings::default();
        if let Some(field) = json.get("CacheFilename") {
            if let Some(v) = field.as_str() {
                settings.cache_filename = v.to_string();
            } else {
                return Err(Error::InvalidSettingsField("CacheFilename".to_string()));
            }
        }
        if let Some(field) = json.get("CacheUnpackDir") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_unpack_directory = v.to_string();
            } else {
                return Err(Error::InvalidSettingsField("CacheUnpackDir".to_string()));
            }
        }
        if let Some(field) = json.get("CacheArchiveName") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_archive_name = v.to_string();
            } else {
                return Err(Error::InvalidSettingsField("CacheArchiveName".to_string()));
            }
        }
        if let Some(field) = json.get("CacheRDFDownloadLink") {
            if let Some(v) = field.as_str() {
                settings.cache_rdf_download_link = v.to_string();
            } else {
                return Err(Error::InvalidSettingsField(
                    "CacheRDFDownloadLink".to_string(),
                ));
            }
        }
        if let Some(field) = json.get("TextFilesCacheFolder") {
            if let Some(v) = field.as_str() {
                settings.text_files_cache_folder = v.to_string();
            } else {
                return Err(Error::InvalidSettingsField(
                    "TextFilesCacheFolder".to_string(),
                ));
            }
        }
        Ok(settings)
    }
}
