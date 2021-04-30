#[derive(Debug)]
pub struct Movie {
    pub name: String,
    pub youtube_id: String,
    pub genre: Genre,
}

#[derive(Debug)]
pub enum Genre {
    Unknown,
}

impl Default for Genre {
    fn default() -> Self {
        Genre::Unknown
    }
}