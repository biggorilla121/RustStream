use axum::{
    extract::{Path, Query, State},
    http,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use http::HeaderMap;
use serde::Deserialize;
use sqlx::Pool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;

mod api;
mod auth;
mod config;
mod db;
mod error;
mod models;
mod tmdb;
mod vidking;
mod templates;
mod onboarding;

use crate::auth::{AuthManager, Session, SessionStore};
use crate::config::Config;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<sqlx::Sqlite>,
    pub tmdb: tmdb::TmdbClient,
    pub vidking: Arc<vidking::VidkingClient>,
    pub auth: Arc<AuthManager>,
    pub sessions: Arc<SessionStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting streaming app...");

    onboarding::maybe_run_onboarding()?;

    let config = Config::new()?;
    info!("Configuration loaded");

    let db_pool = db::init_db(&config.database_url).await?;
    info!("Database initialized");

    let auth_manager = AuthManager::new(db_pool.clone());
    auth_manager.init_local_user().await?;
    
    let session_store = SessionStore::new(db_pool.clone());

    let tmdb_client = tmdb::TmdbClient::new(&config.tmdb_api_key)?;
    info!("TMDB client initialized");

    let vidking_client = vidking::VidkingClient::new()?;
    info!("Vidking client initialized");

    let state = AppState {
        db: db_pool,
        tmdb: tmdb_client,
        vidking: Arc::new(vidking_client),
        auth: Arc::new(auth_manager),
        sessions: Arc::new(session_store),
    };

    let app = Router::new()
        .route("/", get(home_page))
        .route("/search", get(search_page))
        .route("/history", get(watch_history_page))
        .route("/movie/:id", get(movie_detail_page))
        .route("/tv/:id", get(tv_detail_page))
        .route("/player/:media_type/:id", get(player_page))
        .route("/api/progress", post(api_update_progress))
        .nest("/api", api::routes(state.clone()))
        .nest_service("/static", ServeDir::new("app/static"))
        .with_state(state);

    let addr: SocketAddr = format!("127.0.0.1:{}", config.port).parse()?;
    info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_session(state: &AppState, _headers: &HeaderMap) -> Option<Session> {
    state.auth.get_local_session().await.ok()
}

async fn home_page(State(state): State<AppState>, headers: HeaderMap) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let username = session.as_ref().map(|s| s.username.as_str());
    let trending = state.tmdb.get_trending("movie", "week").await?;
    let popular_tv = state.tmdb.get_popular_tv(1).await?;
    let trending_searches = state.tmdb.get_trending_searches().await;
    
    let html = templates::render_home(username, &trending.results, &popular_tv.results, &trending_searches);
    Ok(Html(html))
}


#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    genre: Option<String>,
    year: Option<i32>,
    min_rating: Option<f64>,
    sort_by: Option<String>,
}

async fn search_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SearchQuery>,
) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let username = session.as_ref().map(|s| s.username.as_str());
    let query = params.q.clone().unwrap_or_default();
    
    let has_filters = params.genre.is_some() || params.year.is_some() || params.min_rating.is_some();
    
    let results = if has_filters {
        state.tmdb.search_advanced(
            &query,
            None,
            params.year,
            params.genre.as_deref(),
            params.min_rating,
            &params.sort_by.unwrap_or_else(|| "popularity.desc".to_string()),
            1,
        ).await?.results
    } else if query.len() >= 2 {
        let mut results = state.tmdb.search(&query, 1).await?.results;
        results.retain(|r| r.media_type != "person");
        results
    } else {
        vec![]
    };
    
    let genres = state.tmdb.get_genres().await?;
    let html = templates::render_search(username, &query, &results, &genres);
    Ok(Html(html))
}

async fn watch_history_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let username = session.as_ref().map(|s| s.username.as_str());
    
    let history = match session {
        Some(ref s) => state.auth.get_watch_history(s.user_id).await?,
        None => vec![],
    };
    
    let html = templates::render_watch_history(username, &history);
    Ok(Html(html))
}

#[derive(Deserialize)]
struct ProgressRequest {
    tmdb_id: i64,
    media_type: String,
    progress: f64,
    current_time: f64,
    duration: f64,
    season: Option<i64>,
    episode: Option<i64>,
    title: String,
    poster_path: Option<String>,
    episode_title: Option<String>,
    completed: bool,
}

async fn api_update_progress(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<ProgressRequest>,
) -> Result<Json<()>, AppError> {
    let session = get_session(&state, &headers).await;
    
    if let Some(s) = session {
        state.auth.add_to_watch_history(
            s.user_id,
            data.tmdb_id,
            &data.media_type,
            &data.title,
            data.poster_path.as_deref(),
            data.season,
            data.episode,
            data.episode_title.as_deref(),
        ).await?;
        
        state.auth.update_watch_progress(
            s.user_id,
            data.tmdb_id,
            &data.media_type,
            data.current_time as i64,
            data.completed,
            data.season,
            data.episode,
        ).await?;
    }
    
    Ok(Json(()))
}

async fn movie_detail_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let username = session.as_ref().map(|s| s.username.as_str());
    let movie = state.tmdb.get_movie(id).await?;
    let poster_path = movie.poster_path.as_deref();
    let html = templates::render_movie_detail(username, &movie);
    Ok(Html(html))
}

async fn tv_detail_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let username = session.as_ref().map(|s| s.username.as_str());
    let show = state.tmdb.get_tv_show(id).await?;
    let poster_path = show.poster_path.as_deref();
    let html = templates::render_tv_detail(username, &show);
    Ok(Html(html))
}

#[derive(Deserialize)]
struct PlayerQuery {
    #[serde(default)]
    season: Option<i64>,
    #[serde(default)]
    episode: Option<i64>,
}

async fn player_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((media_type, id)): Path<(String, i64)>,
    Query(params): Query<PlayerQuery>,
) -> Result<Html<String>, AppError> {
    let session = get_session(&state, &headers).await;
    let is_admin = false;
    let username = session.as_ref().map(|s| s.username.as_str());
    
    let (title, poster_path) = if media_type == "movie" {
        let movie = state.tmdb.get_movie(id).await?;
        (movie.title, movie.poster_path)
    } else {
        let show = state.tmdb.get_tv_show(id).await?;
        (show.name, show.poster_path)
    };

    let streams = if media_type == "movie" {
        state.vidking.get_movie_streams(id).await?
    } else {
        let season = params.season.ok_or_else(|| AppError::BadRequest("Season required".to_string()))?;
        let episode = params.episode.ok_or_else(|| AppError::BadRequest("Episode required".to_string()))?;
        state.vidking.get_tv_streams(id, season, episode).await?
    };
    
    let html = templates::render_player(username, &title, &media_type, id, poster_path.as_deref(), &streams, is_admin);
    Ok(Html(html))
}
