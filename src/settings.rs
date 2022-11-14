pub struct GutenbergCacheSettings {
    pub TEXT_FILES_CACHE_FOLDER: String, 
}

impl Default for GutenbergCacheSettings {
    fn default() -> GutenbergCacheSettings {
        GutenbergCacheSettings {
            TEXT_FILES_CACHE_FOLDER: "text_cache".to_string(),
        }
    }
}