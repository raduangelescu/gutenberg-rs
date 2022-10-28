use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ParseType {
    Title = 0,
    TitleAlternative,
    Subject,
    Language,
    Author,
    AuthorAlternative,
    Bookshelf,
    FilesLinks,
    FilesType,
    Publisher,
    Rights,
    DateIssued,
    Downloads,
}

impl fmt::Display for ParseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseType::Title => write!(f, "Title"),
            ParseType::TitleAlternative => write!(f, "TitleAlternative"),
            ParseType::Subject => write!(f, "Subject"),
            ParseType::Language => write!(f, "Language"),
            ParseType::Author => write!(f, "Author"),
            ParseType::AuthorAlternative => write!(f, "AuthorAlternative"),
            ParseType::Bookshelf => write!(f, "Bookshelf"),
            ParseType::FilesLinks => write!(f, "FilesLinks"),
            ParseType::FilesType => write!(f, "FilesType"),
            ParseType::Publisher => write!(f, "Publisher"),
            ParseType::Rights => write!(f, "Rights"),
            ParseType::DateIssued => write!(f, "DateIssued"),
            ParseType::Downloads => write!(f, "Downloads"),
        }
    }
}
