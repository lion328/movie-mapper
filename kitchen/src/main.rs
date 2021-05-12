mod error;
mod external;
mod movie;
mod movielens;
mod imdb;

use error::*;
use movie::Movie;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use csv;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use rayon::prelude::*;

lazy_static! {
    static ref MOVIES_JSON_PATH: &'static Path = Path::new("movies.json");
}

fn main() -> Result<()> {
    let mut args = env::args();
    let mut stage = 0;
    let mut save = true;

    while let Some(arg) = args.next() {
        match &*arg {
            "--stage" => stage = args.next().unwrap().parse().unwrap(),
            "--nosave" => save = false,
            _ => {},
        }
    }

    println!("Loading movies...");

    let mut movies = init_movies()?;

    if stage == 0 {
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

        if save {
            println!("Saving movies into disk...");
            save_movies(&movies)?;
        }

        stage += 1;
    }
    
    let movies_selected: Vec<_> = movies.par_iter()
        .filter(|m| m.year >= Some(2000))
        .filter(|m| m.critics_number.is_some())
        .filter(|m| m.critics_number.unwrap() >= 100)
        .map(|m| {
            let mut path = PathBuf::new();
            path.set_file_name(format!("trailer-{}", m.youtube_id));
            path.set_extension("jpg");

            (m, path)
        })
        .collect();

    if stage == 1 {
        let mut movies_selected: Vec<_> = movies_selected.par_iter()
            .filter(|(_, p)| !p.exists())
            .map(|(m, p)| {
                let mut pn = p.clone();
                pn.set_extension("m4a");
                (m, pn)
            })
            .collect();
        
        movies_selected.sort_by_key(|m| !m.0.critics_number.unwrap());

        println!("Downloading {} movie trailers...", movies_selected.len());

        let bar = ProgressBar::new(movies_selected.len() as u64);
        let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
        pool.install(|| {
            movies_selected.par_iter().for_each(|(movie, path)| {
                let result = external::download_youtube_m4a_by_id(&movie.youtube_id, &path)
                    .or_else(|_| {
                        let query = if let Some(year) = movie.year {
                            format!("{} {} movie trailer", movie.name, year)
                        } else {
                            format!("{} movie trailer", movie.name)
                        };

                        external::download_youtube_m4a_by_search(&query, &path)
                    });

                if result.is_err() {
                    eprintln!("Failed to download '{}' ({:?}): {}\n",
                        movie.name, movie.year, result.unwrap_err().to_string());
                    return;
                }

                let audio_len = external::get_audio_length(&path).unwrap();
                if audio_len > 10 * 60 * 1000 {
                    eprintln!("Failed to download '{}' ({:?}): length > 10 minutes\n", movie.name, movie.year);
                } else {
                    external::make_spectrogram(&path, (audio_len as usize / 100, 224)).unwrap();
                }

                fs::remove_file(path).unwrap();
                bar.inc(1);
            });
        });

        stage += 1;
    }

    if stage == 2 {
        let exist_movies: Vec<_> = movies_selected.iter()
            .filter(|(_, p)| p.exists())
            .map(|(m, _)| *m)
            .collect();

        println!("All:\t{}", exist_movies.len());
        println!("Number of movies in each genre:");
        let sorted = count_and_print_genres(&exist_movies, None);

        println!("Selecting movies that have 1 in top 5 genres...");
        let top_5_genres: Vec<_> = sorted.iter().rev().take(5).map(|(g, _)| *g).collect();
        let top_5_movies: Vec<_> = exist_movies.into_iter()
            .map(|m| (m, m.genres.iter()
                .filter(|g| top_5_genres.contains(g))
                .map(|g| *g)
                .collect::<Vec<_>>()))
            .filter(|(_, g)| g.len() > 0)
            .collect();

        println!("Number of selected movies: {}", top_5_movies.len());
        println!("Number of movies in each genre:");
        count_and_print_genres(&top_5_movies.iter().map(|(m, _)| *m).collect::<Vec<_>>(), Some(&top_5_genres));

        if save {
            println!("Saving selected movies to disk...");

            let mut wtr = csv::Writer::from_path("movies_selected.csv")?;
            let mut headers: Vec<_> = top_5_genres.iter()
                .map(|m| m.to_string())
                .collect();
            headers.insert(0, "Id".to_owned());
            headers.insert(1, "Name".to_owned());
            wtr.write_record(headers)?;
    
            for (movie, genres) in &top_5_movies {
                wtr.write_field(&movie.youtube_id)?;
                wtr.write_field(&movie.name)?;
    
                for genre in &top_5_genres {
                    if genres.contains(genre) {
                        wtr.write_field("1")?;
                    } else {
                        wtr.write_field("0")?;
                    }
                }
    
                wtr.write_record(None::<&[u8]>)?;
            }
        }
    }

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
    if movie.genres.len() == 0
        || movie.critics_number.is_none()
        || movie.metacritic_score.is_none() {
        let imdb_id = imdb::search(&movie.name, movie.year)?;
        let page = imdb::get_page(imdb_id)?;

        movie.genres = imdb::get_genres(&page)?;
        movie.critics_number = imdb::get_critics_number(&page).ok(); // ignore error
        movie.metacritic_score = imdb::get_metascore(&page).ok();
    }

    Ok(())
}

fn count_and_print_genres(movies: &[&movie::Movie], selected_genre: Option<&[movie::Genre]>) -> Vec<(movie::Genre, usize)> {
    let mut map: HashMap<movie::Genre, usize, _> = HashMap::new();
    let mut n = 0;

    for movie in movies {
        for genre in &movie.genres {
            if let Some(includes) = selected_genre {
                if !includes.contains(genre) {
                    continue;
                }
            }

            let mut next = 0;
            if let Some(old) = map.get(genre) {
                next = *old;
            }
            next += 1;
            map.insert(*genre, next);
        }
        n += 1;
    }

    let mut sorted: Vec<_> = map.into_iter().collect();
    sorted.sort_by_key(|(_, i)| *i);

    sorted.iter().rev().for_each(|(g, i)| println!("{}\t{:02.2}\t{:?}", i, *i as f32 * 100.0 / n as f32, g));

    sorted
}