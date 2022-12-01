#[derive(Debug, Copy, Clone)]
pub struct GutenbergFileEntry {
    pub file_link_id: i32,
    pub file_type_id: i32,
}

pub struct Book {
    pub publisher_id: i32,
    pub title_id: i32,
    pub rights_id: i32,
    pub gutenberg_book_id: usize,
    pub date_issued: String,
    pub num_downloads: i32,

    pub language_ids: Vec<usize>,
    pub subject_ids: Vec<usize>,
    pub author_ids: Vec<usize>,
    pub bookshelf_ids: Vec<usize>,

    pub files: Vec<GutenbergFileEntry>,
}
