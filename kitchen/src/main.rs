mod error;
mod movie;
mod movielens;
mod imdb;

fn main() {
    println!("{:?}", movielens::download().unwrap());
}
