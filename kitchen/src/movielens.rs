use crate::error::{Error, Result};
use crate::movie::Movie;

use std::io::{Cursor, Read};

use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::get as http_get;

const CSV_NAME: &str = "ml-youtube.csv";
const ZIP_URL: &str = "http://files.grouplens.org/datasets/movielens/ml-20m-youtube.zip";
const ZIP_CHECKSUM_URL: &str = "http://files.grouplens.org/datasets/movielens/ml-20m-youtube.zip.md5";

lazy_static! {
    static ref TITLE_REGEX: Regex = Regex::new(
        r"^(?:([^,]*)|(?:(.*),\s*(.*)))\s*\((\s*.*\s*)\).*$").unwrap();
    static ref TITLE_NO_YEAR_REGEX: Regex = Regex::new(
        r"^(?:([^,]*)|(?:(.*),\s*(.*)))$").unwrap();
}

pub fn download() -> Result<Vec<Movie>> {
    let original_md5 = http_get(ZIP_CHECKSUM_URL)?.text()?.replace("MD5 (ml-youtube.zip) = ", "");
    let original_md5 = original_md5.trim();

    let zip_bytes = http_get(ZIP_URL)?.bytes()?;
    let computed_md5 = format!("{:x}", md5::compute(zip_bytes.clone()));

    if original_md5 != computed_md5 {
        return Err(Error::Checksum(computed_md5, original_md5.to_owned()));
    }

    let reader = Cursor::new(zip_bytes);
    let mut zip = zip::read::ZipArchive::new(reader)?;
    let csv_file = zip.by_name(CSV_NAME)?;
    
    from_reader(csv_file)
}

fn from_reader<T: Read>(reader: T) -> Result<Vec<Movie>> {
    let mut csv_reader = csv::Reader::from_reader(reader);

    let mut movies = vec![];

    for result in csv_reader.records() {
        let record = result?;
        let raw_name = record.get(2).unwrap().trim();

        let (name, year) = extract_name_and_year(raw_name);

        movies.push(Movie {
            year: year,
            name: name,
            youtube_id: record.get(0).unwrap().to_owned(),
            ..Default::default()
        });
    }

    Ok(movies)
}

fn extract_name_and_year(raw_name: &str) -> (String, Option<u32>) {
    let mut name = String::new();
    let mut year_str = String::new();
    let mut year = None;
    let mut is_year = false;
    let mut prev = '\x00';

    for c in raw_name.chars() {
        if c.is_whitespace() && prev.is_whitespace() {
            continue;
        }

        if is_year {
            if c == ')' {
                if let Ok(num) = year_str.parse::<u32>() {
                    year = Some(num);
                    break;
                }
                is_year = false;
                year_str = String::new();
            } else {
                year_str.push(c);
            }
        } else if c == '(' {
            is_year = true;
        } else {
            name.push(c);
        }

        prev = c;
    }

    let mut processed_name = name.trim().to_owned();

    if let Some(pos) = name.rfind(',') {
        let (mut name_part, mut article) = name.split_at(pos);
        name_part = name_part.trim();
        article = &article[1..]; // Remove ','
        article = article.trim();
        
        match &*article.to_lowercase() {
            "a" | "an" | "the" | "le" | "les" => processed_name = format!("{} {}", article, name_part),
            _ => {},
        }
    }

    (processed_name, year)
}

#[cfg(test)]
mod test {
    #[test]
    fn download() {
        let movies = super::download().unwrap();
        let lotr = movies.into_iter()
            .find(|m| m.name == "The Lord of the Rings: The Fellowship of the Ring")
            .unwrap();
        assert_eq!(lotr.year, Some(2001));
    }

    #[test]
    fn extract_name_and_year() {
        assert_eq!(
            super::extract_name_and_year("Kenji Mizoguchi: The Life of a Film Director (Aru eiga-kantoku no shogai) (1975)"),
            ("Kenji Mizoguchi: The Life of a Film Director".to_owned(), Some(1975))
        );
        assert_eq!(
            super::extract_name_and_year("Misérables, Les (1995)"),
            ("Les Misérables".to_owned(), Some(1995))
        );
        assert_eq!(
            super::extract_name_and_year("Honey, I Shrunk the Kids (1989)"),
            ("Honey, I Shrunk the Kids".to_owned(), Some(1989))
        );
    }
}