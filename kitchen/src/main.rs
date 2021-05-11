mod error;
mod movie;
mod movielens;
mod imdb;

use error::*;
use movie::Movie;

use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use indicatif::ProgressBar;
use lazy_static::lazy_static;
use rayon::prelude::*;

lazy_static! {
    static ref MOVIES_JSON_PATH: &'static Path = Path::new("movies.json");
}

fn main() -> Result<()> {
    println!("Loading movies...");

    let mut movies = init_movies()?;

    println!("Filling missing information...");

    let movies_2000: Vec<_> = movies.iter_mut()
        .filter(|m| m.year >= Some(2000))
        .collect();

    let bar = ProgressBar::new(movies_2000.len() as u64);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(16).build().unwrap();
    pool.install(|| {
        movies_2000.into_par_iter().for_each(|movie| {
            let result = fill_movie_info(movie);
            if result.is_err() {
                eprintln!("Failed to fill '{}' ({:?}): {}\n",
                    movie.name, movie.year, result.unwrap_err().to_string());
            }
            bar.inc(1);
        });
    });

    println!("Saving movies into disk...");
    save_movies(&movies)?;

    Ok(())
}

fn init_movies() -> Result<Vec<Movie>> {
    if MOVIES_JSON_PATH.exists() {
        return Ok(serde_json::from_reader(File::open(*MOVIES_JSON_PATH)?)?);
    }

    println!("Downloading MovieLens dataset...");

    let movies = movielens::download()?;
    save_movies(&movies)?;

    Ok(movies)
}

fn save_movies(movies: &Vec<Movie>) -> Result<()> {
    Ok(serde_json::to_writer_pretty(File::create(*MOVIES_JSON_PATH)?, movies)?)
}

fn fill_movie_info(movie: &mut Movie) -> Result<()> {
    let imdb_id = imdb::search(&movie.name, movie.year)?;
    let page = imdb::get_page(imdb_id)?;

    if movie.genres.len() == 0 {
        movie.genres = imdb::get_genres(&page)?;
    }

    if movie.critics_number.is_none() {
        movie.critics_number = imdb::get_critics_number(&page).ok(); // ignore error
    }

    if movie.metacritic_score.is_none() {
        movie.metacritic_score = imdb::get_metascore(&page).ok();
    }

    Ok(())
}