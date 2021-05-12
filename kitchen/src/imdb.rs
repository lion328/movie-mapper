use crate::error::*;
use crate::movie::Genre;

use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{blocking::get as http_get, Url, IntoUrl};
use kuchiki::{traits::*, NodeRef};

const IMDB_BASE_URL_STR: &str = "https://www.imdb.com";

lazy_static! {
    static ref CRITICS_NUMBER_REGEX: Regex = Regex::new(r"(\d+) critic").unwrap();
    static ref TITLE_YEAR_TYPE_REGEX: Regex = Regex::new(r"\((\d{4})\)\s*(?:\((.*)\))?\s*$").unwrap();
    static ref TITLE_HREF_ID_REGEX: Regex = Regex::new(r"/tt(\d+)/").unwrap();
}

fn parse_page<T: IntoUrl>(url: T) -> Result<NodeRef> {
    let html = http_get(url)?.text()?;
    Ok(kuchiki::parse_html().one(html))
}

pub fn search(name: &str, year: Option<u32>) -> Result<u32> {
    search_impl(name, year, false)
}

#[cfg(test)]
pub fn search_exact(name: &str, year: Option<u32>) -> Result<u32> {
    search_impl(name, year, true)
}

fn search_impl(name: &str, year: Option<u32>, exact: bool) -> Result<u32> {
    let url = Url::parse_with_params(&format!("{}/find", IMDB_BASE_URL_STR), &[
        ("q", name),
        ("s", "tt"),
        ("exact", &exact.to_string()),
    ]).map_err(|e| Error::ParseError(format!("url: {:?}", e)))?;

    let mut candidate: Option<(u32, u32)> = None;

    let document = parse_page(url)?;
    for css_match in document.select(".result_text > a")? {
        let node_ref = css_match.as_node();
        let mut siblings = node_ref.inclusive_following_siblings();

        let (id, searched_name_ref) = {
            let node = siblings.next().unwrap();
            let elem = node.as_element().unwrap();
            let attrs = elem.attributes.borrow();
            let href = attrs.get("href").unwrap();

            let captures = TITLE_HREF_ID_REGEX.captures(href);
            if captures.is_none() {
                continue;
            }

            let id_str = captures.unwrap().get(1).unwrap().as_str();
            let id = id_str.parse().map_err(|_| Error::ParseError(id_str.to_owned()))?;

            let text_node = node.first_child().unwrap();
            let text = text_node.as_text().unwrap().clone();

            (id, text)
        };

        let node = siblings.next().unwrap();
        let text = node.as_text().unwrap();
        let s = text.borrow();

        let captures = TITLE_YEAR_TYPE_REGEX.captures(&s);

        let mut searched_year = 0;
        let mut title_type = None;

        if let Some(captures) = captures {
            let year_str = captures.get(1).unwrap().as_str();
            searched_year = year_str.parse().map_err(|_| Error::ParseError(year_str.to_owned()))?;
            title_type = captures.get(2).map(|t| t.as_str());
        };

        if title_type.map(|t|
            t.contains("TV")
            && !t.contains("TV Movie")
            && !t.contains("TV Special")
        ) == Some(true) {
            continue;
        }

        let mut loss = 1;
        if !exact && name.trim().to_lowercase() != searched_name_ref.borrow().trim().to_lowercase() {
            loss = 2;
        }

        if let Some(year) = year {
            loss *= (year as i64 - searched_year as i64).abs() as u32;
    
            if loss == 0 {
                return Ok(id)
            }

            if candidate.map(|c| c.0 > loss) != Some(false) {
                candidate = Some((loss, id));
            }
        }
    }

    if candidate.map(|c| c.0) >= Some(4) {
        Err(format!("movie search found no result with loss = {}", candidate.unwrap().0))?
    } else if let Some(c) = candidate {
        Ok(c.1)
    } else {
        Err("movie search found no result")?
    }
}

pub fn get_page(id: u32) -> Result<NodeRef> {
    parse_page(get_url(id))
}

pub fn get_genres(document: &NodeRef) -> Result<Vec<Genre>> {
    for css_match in document.select("h4")? {
        let as_node = css_match.as_node();

        if let Some(inner) = as_node.first_child() {
            let text = inner.as_text().unwrap().borrow();

            if *text != "Genres:" {
                continue;
            }
        }

        if let Some(parent) = as_node.ancestors().nth(0) {
            let mut result = vec![];

            for genre_link in parent.select("a")? {
                let inner = genre_link.as_node().first_child().unwrap();
                let text = inner.as_text().unwrap().borrow();
                result.push(Genre::from_text(text.trim())?);
            }

            return Ok(result);
        }
    }

    Err(Error::UnknownGenre("no genre found".to_owned()))
}

pub fn get_metascore(document: &NodeRef) -> Result<u8> {
    for css_match in document.select(".metacriticScore > span")? {
        let as_node = css_match.as_node();

        if let Some(inner) = as_node.first_child() {
            let text = inner.as_text().unwrap().borrow();
            let score = text.parse().map_err(|_| Error::ParseError(text.clone()));
            return score;
        }
    }

    Err("missing metacritic score")?
}

pub fn get_critics_number(document: &NodeRef) -> Result<u32> {
    for css_match in document.select(".titleReviewBarItem a[href^=externalreviews]")? {
        let as_node = css_match.as_node();

        if let Some(inner) = as_node.first_child() {
            let text = inner.as_text().unwrap().borrow();

            let captures = CRITICS_NUMBER_REGEX.captures(&text).unwrap();
            let n_str = captures.get(1).unwrap().as_str();
            let n = n_str.parse().map_err(|_| Error::ParseError(n_str.to_owned()))?;
            
            return Ok(n)
        }
    }

    Err("missing critics number")?
}

fn get_url(id: u32) -> String {
    format!("{}/title/tt{:07}", IMDB_BASE_URL_STR, id)
}

#[cfg(test)]
mod tests {
    const LORD_OF_THE_RINGS_2001_ID: u32 = 120737;

    #[test]
    fn get_url() {
        assert_eq!(super::get_url(LORD_OF_THE_RINGS_2001_ID), "https://www.imdb.com/title/tt0120737");
    }

    #[test]
    fn search() {
        assert_eq!(LORD_OF_THE_RINGS_2001_ID,
            super::search("Lord of the Rings: The Fellowship of the Ring, The", Some(2001)).unwrap());

        assert_eq!(127349, super::search_exact("Waking the Dead", Some(2000)).unwrap());
        assert_eq!(344864, super::search_exact("Atlantis: Milo's Return", Some(2003)).unwrap());
        assert_eq!(1363127, super::search_exact("Northern Lights", Some(2009)).unwrap());
        assert_eq!(1165253, super::search_exact("Dream", Some(2008)).unwrap());
        assert_eq!(1439235, super::search_exact("Jim Jefferies: I Swear to God", Some(2008)).unwrap());
    }

    #[test]
    fn get_infos() {
        use crate::movie::Genre::*;

        let page = super::get_page(LORD_OF_THE_RINGS_2001_ID).unwrap();
        let genres = super::get_genres(&page).unwrap();
        assert_eq!(genres, vec![Action, Adventure, Drama, Fantasy]);

        let score = super::get_metascore(&page).unwrap();
        assert!(score >= 90); // Unlikely to be < 90. It's TLOTR!

        let critics = super::get_critics_number(&page).unwrap();
        assert!(critics >= 330);
    }
}