use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

#[derive(Debug, Clone)]
pub struct TmdbClient {
    client: Client,
    api_key: String,
}

impl TmdbClient {
    pub fn new(api_key: &str) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.to_string(),
        })
    }

    fn auth_header(&self) -> String {
        if self.api_key.starts_with("Bearer ") {
            self.api_key.clone()
        } else {
            format!("Bearer {}", self.api_key)
        }
    }

    pub async fn search(&self, query: &str, page: i32) -> anyhow::Result<SearchResponse> {
        let url = format!("{}/search/multi", TMDB_BASE_URL);
        
        debug!("Searching TMDB for: {}", query);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[
                ("query", query),
                ("page", &page.to_string()),
                ("include_adult", "false"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("TMDB search error: {}", error_text);
            return Err(anyhow::anyhow!("TMDB API error: {}", error_text));
        }

        let search_results: SearchResponse = response.json().await?;
        Ok(search_results)
    }

    pub async fn search_advanced(
        &self,
        query: &str,
        media_type: Option<&str>,
        year: Option<i32>,
        genre: Option<&str>,
        min_rating: Option<f64>,
        sort_by: &str,
        page: i32,
    ) -> anyhow::Result<SearchResponse> {
        let url = format!("{}/discover/movie", TMDB_BASE_URL);
        
        debug!("Advanced search: query={}, type={:?}, year={:?}, genre={:?}, min_rating={:?}, sort={}",
               query, media_type, year, genre, min_rating, sort_by);
        
        let mut query_params: Vec<(&str, String)> = Vec::new();
        
        if let Some(q) = query.strip_prefix("genre:") {
            let genre_id = get_genre_id(q);
            query_params.push(("with_genres", genre_id.to_string()));
        } else if let Some(q) = query.strip_prefix("actor:") {
            let person_id = self.search_person(q).await?;
            if person_id > 0 {
                query_params.push(("with_cast", person_id.to_string()));
            }
        } else if let Some(q) = query.strip_prefix("director:") {
            let person_id = self.search_person(q).await?;
            if person_id > 0 {
                query_params.push(("with_crew", person_id.to_string()));
            }
        } else if !query.is_empty() {
            query_params.push(("query", query.to_string()));
        }
        
        if let Some(mt) = media_type {
            query_params.push(("media_type", mt.to_string()));
        }
        
        if let Some(y) = year {
            query_params.push(("year", y.to_string()));
            query_params.push(("primary_release_year", y.to_string()));
        }
        
        if let Some(rating) = min_rating {
            query_params.push(("vote_average.gte", rating.to_string()));
        }
        
        query_params.push(("sort_by", sort_by.to_string()));
        query_params.push(("page", page.to_string()));
        query_params.push(("include_adult", "false".to_string()));
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("TMDB advanced search error: {}", error_text);
            return Err(anyhow::anyhow!("TMDB API error: {}", error_text));
        }

        let search_results: SearchResponse = response.json().await?;
        Ok(search_results)
    }

    async fn search_person(&self, name: &str) -> anyhow::Result<i64> {
        let url = format!("{}/search/person", TMDB_BASE_URL);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[
                ("query", name),
                ("include_adult", "false"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(0);
        }

        #[derive(Debug, Deserialize)]
        struct PersonResult {
            pub id: i64,
            pub name: String,
        }
        
        #[derive(Debug, Deserialize)]
        struct PersonResponse {
            pub results: Vec<PersonResult>,
        }

        let person_results: PersonResponse = response.json().await?;
        Ok(person_results.results.first().map(|p| p.id).unwrap_or(0))
    }

    pub async fn get_genres(&self) -> anyhow::Result<Vec<Genre>> {
        let url = format!("{}/genre/movie/list", TMDB_BASE_URL);
        
        #[derive(Debug, Deserialize)]
        struct GenreResponse {
            pub genres: Vec<Genre>,
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let genre_response: GenreResponse = response.json().await?;
        Ok(genre_response.genres)
    }

    pub async fn get_trending_searches(&self) -> Vec<SearchResult> {
        let trending_movies = self.get_trending("movie", "day").await.ok().map(|r| r.results).unwrap_or_default();
        let trending_tv = self.get_trending("tv", "day").await.ok().map(|r| r.results).unwrap_or_default();
        
        let mut combined = trending_movies;
        combined.extend(trending_tv);
        combined.truncate(10);
        combined
    }

    pub async fn get_movie(&self, id: i64) -> anyhow::Result<MovieDetail> {
        let url = format!("{}/movie/{}", TMDB_BASE_URL, id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[("append_to_response", "credits,similar")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch movie details"));
        }

        let movie: MovieDetail = response.json().await?;
        Ok(movie)
    }

    pub async fn get_tv_show(&self, id: i64) -> anyhow::Result<TvShowDetail> {
        let url = format!("{}/tv/{}", TMDB_BASE_URL, id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[("append_to_response", "credits,similar")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch TV show details"));
        }

        let show: TvShowDetail = response.json().await?;
        Ok(show)
    }

    pub async fn get_popular_movies(&self, page: i32) -> anyhow::Result<MovieListResponse> {
        let url = format!("{}/movie/popular", TMDB_BASE_URL);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[("page", page.to_string())])
            .send()
            .await?;

        Ok(response.json().await?)
    }

    pub async fn get_popular_tv(&self, page: i32) -> anyhow::Result<TvListResponse> {
        let url = format!("{}/tv/popular", TMDB_BASE_URL);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[("page", page.to_string())])
            .send()
            .await?;

        Ok(response.json().await?)
    }

    pub async fn get_trending(&self, media_type: &str, time_window: &str) -> anyhow::Result<SearchResponse> {
        let url = format!("{}/trending/{}/{}", TMDB_BASE_URL, media_type, time_window);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        Ok(response.json().await?)
    }

    pub fn get_poster_url(&self, path: Option<&str>, size: &str) -> Option<String> {
        path.map(|p| format!("{}/{}{}", TMDB_IMAGE_BASE, size, p))
    }

    pub fn get_backdrop_url(&self, path: Option<&str>, size: &str) -> Option<String> {
        path.map(|p| format!("{}/{}{}", TMDB_IMAGE_BASE, size, p))
    }
}

fn get_genre_id(genre_name: &str) -> i64 {
    let genre_map: Vec<(&str, i64)> = vec![
        ("action", 28),
        ("adventure", 12),
        ("animation", 16),
        ("comedy", 35),
        ("crime", 80),
        ("documentary", 99),
        ("drama", 18),
        ("family", 10751),
        ("fantasy", 14),
        ("history", 36),
        ("horror", 27),
        ("music", 10402),
        ("mystery", 9648),
        ("romance", 10749),
        ("sci-fi", 878),
        ("thriller", 53),
        ("war", 10752),
        ("western", 37),
    ];
    
    let normalized = genre_name.to_lowercase().replace(' ', "-");
    genre_map.iter()
        .find(|(name, _)| *name == normalized || name.replace("-", " ") == normalized)
        .map(|(_, id)| *id)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResponse {
    pub page: i32,
    pub results: Vec<SearchResult>,
    pub total_pages: i32,
    pub total_results: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: i64,
    #[serde(default)]
    pub adult: bool,
    #[serde(default)]
    pub media_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub first_air_date: Option<String>,
    #[serde(default)]
    pub vote_average: f64,
    #[serde(default)]
    pub vote_count: i64,
    #[serde(default)]
    pub genre_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub popularity: f64,
    #[serde(default)]
    pub original_language: Option<String>,
    #[serde(default)]
    pub origin_country: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovieDetail {
    pub id: i64,
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub vote_average: f64,
    pub vote_count: i64,
    pub genres: Vec<Genre>,
    pub credits: Option<Credits>,
    pub similar: Option<SimilarMovies>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvShowDetail {
    pub id: i64,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub number_of_seasons: Option<i64>,
    pub number_of_episodes: Option<i64>,
    pub vote_average: f64,
    pub vote_count: i64,
    pub genres: Vec<Genre>,
    pub seasons: Vec<SeasonInfo>,
    pub credits: Option<Credits>,
    pub similar: Option<SimilarTvShows>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credits {
    pub cast: Vec<CastMember>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CastMember {
    pub id: i64,
    pub name: String,
    pub character: String,
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimilarMovies {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimilarTvShows {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeasonInfo {
    pub id: i64,
    pub season_number: i64,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub episode_count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovieListResponse {
    pub page: i32,
    pub results: Vec<SearchResult>,
    pub total_pages: i32,
    pub total_results: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvListResponse {
    pub page: i32,
    pub results: Vec<SearchResult>,
    pub total_pages: i32,
    pub total_results: i32,
}