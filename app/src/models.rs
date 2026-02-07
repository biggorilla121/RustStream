use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: i64,
    pub tmdb_id: i64,
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i64,
    pub genres: Vec<String>,
    pub runtime: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShow {
    pub id: i64,
    pub tmdb_id: i64,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i64,
    pub genres: Vec<String>,
    pub number_of_seasons: Option<i64>,
    pub number_of_episodes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: i64,
    pub tmdb_id: i64,
    pub show_id: i64,
    pub season_number: i64,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub air_date: Option<String>,
    pub episode_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub tmdb_id: i64,
    pub season_id: i64,
    pub episode_number: i64,
    pub name: String,
    pub overview: Option<String>,
    pub still_path: Option<String>,
    pub air_date: Option<String>,
    pub runtime: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingLink {
    pub id: String,
    pub title: String,
    pub url: String,
    pub quality: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub media_type: String,
    pub title: Option<String>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDetail {
    #[serde(flatten)]
    pub content: Content,
    pub similar: Vec<Content>,
    pub cast: Vec<CastMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    Movie(Movie),
    TvShow(TvShow),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMember {
    pub id: i64,
    pub name: String,
    pub character: String,
    pub profile_path: Option<String>,
}
