use num_derive::FromPrimitive;
use std::fmt;

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum ParseType {
    Title = 0,
    Subject,
    Language,
    Author,
    Bookshelf,
    Files,
    Publisher,
    Rights,
    DateIssued,
    Downloads,
}

impl fmt::Display for ParseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseType::Title => write!(f, "Title"),
            ParseType::Subject => write!(f, "Subject"),
            ParseType::Language => write!(f, "Language"),
            ParseType::Author => write!(f, "Author"),
            ParseType::Bookshelf => write!(f, "Bookshelf"),
            ParseType::Files => write!(f, "FilesLinks"),
            ParseType::Publisher => write!(f, "Publisher"),
            ParseType::Rights => write!(f, "Rights"),
            ParseType::DateIssued => write!(f, "DateIssued"),
            ParseType::Downloads => write!(f, "Downloads"),
        }
    }
}
