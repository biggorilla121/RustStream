use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;

pub async fn init_db(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    info!("Running database migrations...");
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS movies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id INTEGER UNIQUE NOT NULL,
            title TEXT NOT NULL,
            overview TEXT,
            poster_path TEXT,
            backdrop_path TEXT,
            release_date TEXT,
            vote_average REAL DEFAULT 0,
            vote_count INTEGER DEFAULT 0,
            genres TEXT DEFAULT '[]',
            runtime INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tv_shows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id INTEGER UNIQUE NOT NULL,
            name TEXT NOT NULL,
            overview TEXT,
            poster_path TEXT,
            backdrop_path TEXT,
            first_air_date TEXT,
            vote_average REAL DEFAULT 0,
            vote_count INTEGER DEFAULT 0,
            genres TEXT DEFAULT '[]',
            number_of_seasons INTEGER,
            number_of_episodes INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS seasons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id INTEGER UNIQUE NOT NULL,
            show_id INTEGER NOT NULL,
            season_number INTEGER NOT NULL,
            name TEXT,
            overview TEXT,
            poster_path TEXT,
            air_date TEXT,
            episode_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (show_id) REFERENCES tv_shows(id) ON DELETE CASCADE
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id INTEGER UNIQUE NOT NULL,
            season_id INTEGER NOT NULL,
            episode_number INTEGER NOT NULL,
            name TEXT,
            overview TEXT,
            still_path TEXT,
            air_date TEXT,
            runtime INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS streaming_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id INTEGER NOT NULL,
            media_type TEXT NOT NULL,
            season_number INTEGER,
            episode_number INTEGER,
            vidking_id TEXT,
            stream_url TEXT,
            quality TEXT,
            expires_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(tmdb_id, media_type, season_number, episode_number)
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            is_admin BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT UNIQUE NOT NULL,
            user_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            is_admin BOOLEAN DEFAULT 0,
            expires_at INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS watch_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            tmdb_id INTEGER NOT NULL,
            media_type TEXT NOT NULL,
            title TEXT NOT NULL,
            poster_path TEXT,
            season_number INTEGER DEFAULT -1,
            episode_number INTEGER DEFAULT -1,
            episode_title TEXT,
            progress_seconds INTEGER DEFAULT 0,
            completed BOOLEAN DEFAULT 0,
            watched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, tmdb_id, media_type, season_number, episode_number)
        )
        "#
    )
    .execute(&pool)
    .await?;

    info!("Database migrations completed");
    
    Ok(pool)
}