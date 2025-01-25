use rocket::serde::{Deserialize, Serialize};
use sqlx::Encode;

#[derive(Deserialize,Serialize)]
pub struct MovieList {
    titles: Vec<String>,
}



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MovieMetadata {
    pub id:  Option<i32>,
    pub title:  Option<String>,
    pub year:  Option<String>,
    rated:  Option<String>,
    released:  Option<String>,
    runtime:  Option<String>,
    genre:  Option<String>,
    director:  Option<String>,
    writer:  Option<String>,
    actors:  Option<String>,
    pub plot:  Option<String>,
    language:  Option<String>,
    country:  Option<String>,
    awards:  Option<String>,
    poster:  Option<String>,
    ratings: Option<Vec<Rating>>,
    metascore:  Option<String>,
    #[serde(rename = "imdbRating")]
    imdb_rating:  Option<String>,
    #[serde(rename = "imdbVotes")]
    imdb_votes:  Option<String>,
    #[serde(rename = "imdbID")]
    pub imdb_id:  Option<String>,
    #[serde(rename = "Type")]
    movie_type:  Option<String>, // Reserved keyword, escaped
    dvd: Option<String>,
    #[serde(rename = "BoxOffice")]
    box_office:  Option<String>,
    production:  Option<String>,
    website:  Option<String>,
    pub response: String,
    #[serde(rename = "Error")]
    error: Option<String>,
    pub source:  Option<String>,
    pub first_letter:  Option<String>,

}

#[derive(Serialize, Deserialize, Debug, Encode, sqlx::Type)]
#[serde(rename_all = "PascalCase")]
struct Rating {
    source: String,
    value: String,
}
