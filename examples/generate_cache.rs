use gutenberg_rs::db_cache::DBCache;
use gutenberg_rs::error::Error;
use gutenberg_rs::settings::GutenbergCacheSettings;
use gutenberg_rs::setup_sqlite;
use gutenberg_rs::sqlite_cache::SQLiteCache;
use gutenberg_rs::text_get::{get_text_by_id, strip_headers};
use serde_json::json;
use std::fs;

// this is a helper function that converts a time (hours:minutes) into spoken english time
fn time_to_text(hour: usize, minute: usize) -> Result<String, Error> {
    let nums = vec![
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
        "twenty one",
        "twenty two",
        "twenty three",
        "twenty four",
        "twenty five",
        "twenty six",
        "twenty seven",
        "twenty eight",
        "twenty nine",
    ];
    match minute {
        0 => Ok(format!("{} o'clock", nums[hour])),
        1 => Ok(format!("one minute past {}", nums[hour])),
        59 => Ok(format!("one minute to {}", nums[hour])),
        15 => Ok(format!("quarter past {}", nums[hour])),
        30 => Ok(format!("half past {}", nums[hour])),
        45 => Ok(format!("quarter to {}", nums[hour])),
        _ => {
            if minute <= 30 {
                Ok(format!("{} minutes past {}", nums[minute], nums[hour]))
            } else if minute > 30 {
                Ok(format!(
                    "{} minutes to {}",
                    nums[60 - minute],
                    nums[(hour % 12) + 1]
                ))
            } else {
                Err(Error::InvalidResult(String::from("bad time")))
            }
        }
    }
}

async fn exec() -> Result<(), Error> {
    // let's do something fun in this example :
    // - create the cache
    // - download some english books from particular shelves
    // - search for a certain time mention in all books
    // - display the paragraph with the time mention

    // here we create the cache settings with the default values
    let settings = GutenbergCacheSettings::default();

    // generate the sqlite cache (this will download, parse and create the db)
    setup_sqlite(&settings, false).await?;

    // we grab the newly create cache
    let mut cache = SQLiteCache::get_cache(&settings).unwrap();

    // we query the cache for our particular interests to get the book ids we need
    let res = cache.query(&json!({
                    "language": "\"en\"",
                    "bookshelve": "'Romantic Fiction',
                    'Astounding Stories','Mystery Fiction','Erotic Fiction',
                    'Mythology','Adventure','Humor','Bestsellers, American, 1895-1923',
                    'Short Stories','Harvard Classics','Science Fiction','Gothic Fiction','Fantasy'",
                }))?;

    // we get the first 10 english books from above categories and concat them into a big pile of text
    let max_number_of_texts = 10;
    let mut big_string = String::from("");
    for (idx, r) in res.iter().enumerate() {
        println!("getting text for gutenberg idx: {}", r);
        let links = cache.get_download_links(vec![*r])?;
        for link in links {
            let text = get_text_by_id(&settings, &link).await?;
            let stripped_text = strip_headers(text);
            big_string.push_str(&stripped_text);
            break;
        }
        if idx >= max_number_of_texts {
            break;
        }
    }

    // write the file just so we have it
    let output_filename = "big_file.txt";
    if std::path::Path::new(output_filename).exists() {
        // delete it if it already exists
        fs::remove_file(output_filename)?;
    }

    fs::write(output_filename, &big_string)?;
    // we get the time in words
    let word_time = time_to_text(6, 0)?;
    println!("The time is {}, now lets search the books", &word_time);

    // we find the time in our pile of text and display the paragraph
    let index = big_string.find(&word_time);
    match index {
        Some(found) => {
            // find the whole paragraph where we have the time mentioned
            let search_window_size = 1000;
            let back_search = &big_string[found - search_window_size..found];
            let start_paragraph = match back_search.rfind("\n\r") {
                Some(x) => found + x - search_window_size,
                None => found - search_window_size,
            };
            let end_search = &big_string[found..found + search_window_size];
            let end_paragraph = match end_search.find("\n\r") {
                Some(x) => x + found,
                None => found + search_window_size,
            };

            let slice = &big_string[start_paragraph..end_paragraph];
            print!(
                "{}-{} [{}] {}",
                start_paragraph, end_paragraph, found, slice
            );
        }
        None => {
            println!("could not find text in books")
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    match exec().await {
        Ok(_e) => {}
        Err(_e) => println!("program failed with error: {}", _e.to_string()),
    }
}
