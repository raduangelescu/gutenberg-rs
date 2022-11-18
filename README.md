Gutenberg-RS
========

This package makes filtering and getting information from [Project Gutenberg](http://www.gutenberg.org) easier from Rust. It started as a port for the [python one](https://github.com/raduangelescu/gutenbergpy) but it is now different in multiple ways.

It's target audience is people working in machine learning that need data for their project, but may be freely used by anybody. 

The package:

-   Generates a local cache (of all gutenberg informations) that you can interogate to get book ids. The Local cache may be sqlite (default)
-   Downloads and cleans raw text from gutenberg books

The package has been tested with Rust 1.64.0 on both Windows and Linux It is faster and smaller than the python one.


Usage
=====

Building the sqlite cache
------------------
``` rust
let settings = GutenbergCacheSettings::default();
setup_sqlite(&settings, false).await?;
```
This will use the default settings and build the cache (if it is not already built). It will download the archive from gutenberg, unpack, parse and store the info.
After building the cache you may get it and query it via a helper function or native sqlite queries:

```rust
let mut cache = SQLiteCache::get_cache(&settings).unwrap();
let res = cache.query(&json!({
                    "language": "\"en\"",
                }))?;
```
The helper query function will return book ids which you can then use to get the text like this:
```rust
use gutenberg_rs::sqlite_cache::SQLiteCache;
use gutenberg_rs::text_get::get_text_by_id;
....
 for (idx, r) in res.iter().enumerate() {
        println!("getting text for gutenberg idx: {}", r);
        let links = cache.get_download_links(vec![*r])?;
        for link in links {
            let res = get_text_by_id(&settings, &link).await.unwrap();
        }
```
The above code will download the book text by id and cache it locally so the next time you need it it will be faster.
You may also strip the headers of text using 
```rust
...
let res = get_text_by_id(&settings, &link).await.unwrap();
let only_content = strip_headers(res)
```
You may find more in the examples folder.

for even better control you may set the GutenbergCacheSettings:

-   *CacheFilename*
-   *CacheUnpackDir*
-   *CacheArchiveName*
-   *CacheRDFDownloadLink*
-   *TextFilesCacheFolder*

``` rust
//example
let mut settings = GutenbergCacheSettings::default();
settings.CacheFilename = "testcachename.db".to_string();
```

The rust version of this library is faster than the python one but the increase is not ten-fold as it could have been as the bottleneck is probably hdd speed (for parsing) and download speed (for getting the content).

Standard query fields:
-   language
-   author
-   type
-   title
-   subject
-   publisher
-   bookshelve
-   downloadtype

