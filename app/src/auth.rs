use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{Pool, Sqlite};
use tracing::info;

pub const SESSION_SECRET: &[u8] = b"your-32-byte-secret-key-change-me-in-prod!";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: i64,
    pub username: String,
    pub is_admin: bool,
    pub expires_at: i64,
}

#[derive(Debug)]
pub struct SessionStore {
    db: Pool<Sqlite>,
}

impl SessionStore {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn create_session(&self, user_id: i64, username: &str, is_admin: bool) -> anyhow::Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = (Utc::now() + Duration::days(7)).timestamp();
        
        let signature = self.create_signature(&session_id, user_id, expires_at);
        let session_token = format!("{}.{}", session_id, signature);
        
        sqlx::query(
            "INSERT INTO sessions (session_id, user_id, username, is_admin, expires_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(username)
        .bind(is_admin)
        .bind(expires_at)
        .execute(&self.db)
        .await?;
        
        info!("Created session for user: {}", username);
        Ok(session_token)
    }

    pub async fn validate_session(&self, session_token: &str) -> anyhow::Result<Option<Session>> {
        let parts: Vec<&str> = session_token.split('.').collect();
        if parts.len() != 2 {
            return Ok(None);
        }
        
        let (session_id, signature) = (parts[0], parts[1]);
        
        let session_row: Option<(String, i64, String, bool, i64)> = sqlx::query_as(
            "SELECT session_id, user_id, username, is_admin, expires_at FROM sessions WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;
        
        if let Some((_, user_id, username, is_admin, expires_at)) = session_row {
            if expires_at < Utc::now().timestamp() {
                sqlx::query("DELETE FROM sessions WHERE session_id = ?")
                    .bind(session_id)
                    .execute(&self.db)
                    .await?;
                return Ok(None);
            }
            
            let expected_signature = self.create_signature(session_id, user_id, expires_at);
            if signature != expected_signature {
                return Ok(None);
            }
            
            Ok(Some(Session {
                id: session_id.to_string(),
                user_id,
                username,
                is_admin,
                expires_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    fn create_signature(&self, session_id: &str, user_id: i64, expires_at: i64) -> String {
        let message = format!("{}.{}.{}", session_id, user_id, expires_at);
        let mut mac = Hmac::<Sha256>::new_from_slice(SESSION_SECRET)
            .expect("HMAC can accept any key size");
        mac.update(message.as_bytes());
        let result = mac.finalize().into_bytes();
        hex::encode(result)
    }
}

pub struct AuthManager {
    db: Pool<Sqlite>,
}

impl AuthManager {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn init_local_user(&self) -> anyhow::Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = 'local')"
        )
        .fetch_one(&self.db)
        .await?;

        if !exists {
            info!("Creating local user...");
            let password_hash = hash("local", DEFAULT_COST)?;
            
            sqlx::query(
                "INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)"
            )
            .bind("local")
            .bind(&password_hash)
            .bind(false)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    pub async fn get_local_session(&self) -> anyhow::Result<Session> {
        let row: Option<(i64, String, bool)> = sqlx::query_as(
            "SELECT id, username, is_admin FROM users WHERE username = 'local' LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await?;

        let (user_id, username, is_admin) = row.ok_or_else(|| anyhow::anyhow!("Local user missing"))?;
        Ok(Session {
            id: "local".to_string(),
            user_id,
            username,
            is_admin,
            expires_at: i64::MAX,
        })
    }

    // User management is intentionally removed for the single-user local mode.

    pub async fn add_to_watch_history(
        &self,
        user_id: i64,
        tmdb_id: i64,
        media_type: &str,
        title: &str,
        poster_path: Option<&str>,
        season_number: Option<i64>,
        episode_number: Option<i64>,
        episode_title: Option<&str>,
    ) -> anyhow::Result<()> {
        let season_num = season_number.unwrap_or(-1);
        let episode_num = episode_number.unwrap_or(-1);
        
        sqlx::query(
            r#"
            INSERT INTO watch_history 
            (user_id, tmdb_id, media_type, title, poster_path, season_number, episode_number, episode_title)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, tmdb_id, media_type, season_number, episode_number)
            DO UPDATE SET watched_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(user_id)
        .bind(tmdb_id)
        .bind(media_type)
        .bind(title)
        .bind(poster_path)
        .bind(season_num)
        .bind(episode_num)
        .bind(episode_title)
        .execute(&self.db)
        .await?;
        
        Ok(())
    }

    pub async fn get_watch_history(&self, user_id: i64) -> anyhow::Result<Vec<WatchHistoryItem>> {
        let items: Vec<WatchHistoryItem> = sqlx::query_as(
            r#"
            SELECT id, user_id, tmdb_id, media_type, title, poster_path, 
                   season_number, episode_number, episode_title, progress_seconds, 
                   completed, watched_at
            FROM watch_history
            WHERE user_id = ?
            ORDER BY watched_at DESC
            LIMIT 50
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;
        
        Ok(items)
    }

    pub async fn update_watch_progress(
        &self,
        user_id: i64,
        tmdb_id: i64,
        media_type: &str,
        progress_seconds: i64,
        completed: bool,
        season_number: Option<i64>,
        episode_number: Option<i64>,
    ) -> anyhow::Result<()> {
        let season_num = season_number.unwrap_or(-1);
        let episode_num = episode_number.unwrap_or(-1);
        
        sqlx::query(
            r#"
            UPDATE watch_history 
            SET progress_seconds = ?, completed = ?, watched_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND tmdb_id = ? AND media_type = ?
            AND season_number = ?
            AND episode_number = ?
            "#
        )
        .bind(progress_seconds)
        .bind(completed)
        .bind(user_id)
        .bind(tmdb_id)
        .bind(media_type)
        .bind(season_num)
        .bind(episode_num)
        .execute(&self.db)
        .await?;
        
        Ok(())
    }

    pub async fn remove_from_watch_history(&self, user_id: i64, history_id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM watch_history WHERE id = ? AND user_id = ?")
            .bind(history_id)
            .bind(user_id)
            .execute(&self.db)
            .await?;
            
        Ok(())
    }

    pub async fn clear_watch_history(&self, user_id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM watch_history WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.db)
            .await?;
            
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WatchHistoryItem {
    pub id: i64,
    pub user_id: i64,
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub poster_path: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub episode_title: Option<String>,
    pub progress_seconds: i64,
    pub completed: bool,
    pub watched_at: String,
}
