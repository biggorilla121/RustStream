use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    AppState,
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/movies/popular", get(get_popular_movies))
        .route("/tv/popular", get(get_popular_tv))
        .route("/trending/:media_type/:time_window", get(get_trending))
        .route("/search", get(search))
        .route("/movie/:id", get(get_movie_detail))
        .route("/tv/:id", get(get_tv_detail))
        .route("/movie/:id/streams", get(get_movie_streams))
        .route("/tv/:id/streams", get(get_tv_streams))
        .with_state(state)
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_page")]
    page: i32,
}

fn default_page() -> i32 {
    1
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<crate::tmdb::SearchResponse>, AppError> {
    let results = state.tmdb.search(&params.q, params.page).await?;
    Ok(Json(results))
}

async fn get_popular_movies(
    State(state): State<AppState>,
) -> Result<Json<crate::tmdb::MovieListResponse>, AppError> {
    let movies = state.tmdb.get_popular_movies(1).await?;
    Ok(Json(movies))
}

async fn get_popular_tv(
    State(state): State<AppState>,
) -> Result<Json<crate::tmdb::TvListResponse>, AppError> {
    let shows = state.tmdb.get_popular_tv(1).await?;
    Ok(Json(shows))
}

async fn get_trending(
    State(state): State<AppState>,
    Path((media_type, time_window)): Path<(String, String)>,
) -> Result<Json<crate::tmdb::SearchResponse>, AppError> {
    let trending = state.tmdb.get_trending(&media_type, &time_window).await?;
    Ok(Json(trending))
}

async fn get_movie_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<crate::tmdb::MovieDetail>, AppError> {
    let movie = state.tmdb.get_movie(id).await?;
    Ok(Json(movie))
}

async fn get_tv_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<crate::tmdb::TvShowDetail>, AppError> {
    let show = state.tmdb.get_tv_show(id).await?;
    Ok(Json(show))
}

#[derive(Deserialize)]
struct StreamQuery {
    #[serde(default)]
    season: Option<i64>,
    #[serde(default)]
    episode: Option<i64>,
}

async fn get_movie_streams(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<crate::vidking::StreamSource>>, AppError> {
    let streams = state.vidking.get_movie_streams(id).await?;
    Ok(Json(streams))
}

async fn get_tv_streams(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<StreamQuery>,
) -> Result<Json<Vec<crate::vidking::StreamSource>>, AppError> {
    let season = params.season.ok_or_else(|| AppError::BadRequest("Season required".to_string()))?;
    let episode = params.episode.ok_or_else(|| AppError::BadRequest("Episode required".to_string()))?;
    
    let streams = state.vidking.get_tv_streams(id, season, episode).await?;
    Ok(Json(streams))
}