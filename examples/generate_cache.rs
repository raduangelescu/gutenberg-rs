use gutenberg_rs::db_cache::DBCache;
use gutenberg_rs::settings::GutenbergCacheSettings;
use gutenberg_rs::setup_sqlite;
use gutenberg_rs::sqlite_cache::SQLiteCache;
use gutenberg_rs::text_get::get_text_by_id;
use gutenberg_rs::error::Error;
use serde_json::json;

async fn exec() -> Result<(), Error> {
    let settings = GutenbergCacheSettings::default();
    setup_sqlite(&settings, false).await?;

    let mut cache = SQLiteCache::get_cache(&settings).unwrap();
    let res = cache.query(&json!({
                    "language": "\"en\"",
                }))?;

    for (idx, r) in res.iter().enumerate() {
        println!("getting text for gutenberg idx: {}", r);
        let links = cache.get_download_links(vec![*r])?;
        for link in links {
            let res = get_text_by_id(&settings, &link).await.unwrap();
        }
        if idx > 20 {
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    match(exec().await) {
        Ok(e) => {},
        Err(e) => println!("program failed with error: {}", e.to_string()),
    }
}
