use crate::error::*;

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Movie {
    pub name: String,
    pub youtube_id: String,
    pub year: Option<u32>,
    pub genres: Vec<Genre>,
    pub critics_number: Option<u32>,
    pub metacritic_score: Option<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genre {
    Fantasy,
    Comedy,
    Adventure,
    Family,
    Animation,
    Drama,
    Action,
    SciFi,
    Romance,
    Short,
    Thriller,
    Mystery,
    Horror,
    Music,
    Crime,
    Musical,
    GameShow,
    History,
    War,
    Sport,
    TalkShow,
    Documentary,
    RealityTV,
    Biography,
    Western,
    News,
    Adult,
    FilmNoir,
}

impl Genre {
    pub fn from_text(text: &str) -> Result<Genre> {
        use Genre::*;

        Ok(match text {
            "Fantasy" => Fantasy,
            "Comedy" => Comedy,
            "Adventure" => Adventure,
            "Family" => Family,
            "Animation" => Animation,
            "Drama" => Drama,
            "Action" => Action,
            "Sci-Fi" => SciFi,
            "Romance" => Romance,
            "Short" => Short,
            "Thriller" => Thriller,
            "Mystery" => Mystery,
            "Horror" => Horror,
            "Music" => Music,
            "Crime" => Crime,
            "Musical" => Musical,
            "Game-Show" => GameShow,
            "History" => History,
            "War" => War,
            "Sport" => Sport,
            "Talk-Show" => TalkShow,
            "Documentary" => Documentary,
            "Reality-TV" => RealityTV,
            "Biography" => Biography,
            "Western" => Western,
            "News" => News,
            "Adult" => Adult,
            "Film-Noir" => FilmNoir,
            x => return Err(Error::UnknownGenre(x.to_owned())),
        })
    }
}