use gutenberg_rs::settings::GutenbergCacheSettings;
use gutenberg_rs::setup_sqlite;
use gutenberg_rs::sqlite_cache::SQLiteCache;
use gutenberg_rs::db_cache::DBCache;
use gutenberg_rs::textget::{get_text_by_id};

use serde_json::json;
use gutenberg_rs::error::Error;

#[tokio::main]
async fn main() {
    let settings = GutenbergCacheSettings::default();
    match setup_sqlite(&settings).await {
        Ok(cache) => {
        let mut cache = SQLiteCache::get_cache(&settings).unwrap();
        let res = cache.query(&json!({
            "languages": "\"en\"",
        })).unwrap();

        for (idx, r) in res.iter().enumerate() {
            println!("getting text for gutenberg idx: {}", r);
            let links = cache.get_download_links(vec![*r]).unwrap();
            for link in links {
                let res = get_text_by_id(&settings, &link).await.unwrap();
            }
            if idx > 20 {
                break;
            }
        }
        },
        Err(ex) => {
            println!("ERROR = {}", ex);
            return;
        }
    }
}
