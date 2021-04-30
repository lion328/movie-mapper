use crate::error::{Error, Result};
use crate::movie::Movie;

use std::io::Cursor;

use reqwest::blocking::get as http_get;
use md5;
use zip;
use csv;

static CSV_NAME: &str = "ml-youtube.csv";
static ZIP_URL: &str = "http://files.grouplens.org/datasets/movielens/ml-20m-youtube.zip";
static ZIP_CHECKSUM_URL: &str = "http://files.grouplens.org/datasets/movielens/ml-20m-youtube.zip.md5";

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
    let mut csv_reader = csv::Reader::from_reader(csv_file);

    let mut movies = vec![];

    for result in csv_reader.records() {
        let record = result?;

        movies.push(Movie {
            name: record.get(2).unwrap().to_owned(),
            youtube_id: record.get(0).unwrap().to_owned(),
            genre: Default::default(),
        });
    }

    Ok(movies)
}

#[cfg(test)]
mod test {
    #[test]
    fn download() {
        super::download().unwrap();
    }
}