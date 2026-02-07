use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

const VIDKING_BASE_URL: &str = "https://www.vidking.net";

#[derive(Debug, Clone)]
pub struct VidkingClient;

impl VidkingClient {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub fn get_movie_embed_url(&self, tmdb_id: i64, options: &EmbedOptions) -> String {
        let mut url = format!("{}/embed/movie/{}", VIDKING_BASE_URL, tmdb_id);
        url.push_str(&options.to_query_string());
        debug!("Generated movie embed URL: {}", url);
        url
    }

    pub fn get_tv_embed_url(&self, tmdb_id: i64, season: i64, episode: i64, options: &EmbedOptions) -> String {
        let mut url = format!("{}/embed/tv/{}/{}/{}", VIDKING_BASE_URL, tmdb_id, season, episode);
        url.push_str(&options.to_query_string());
        debug!("Generated TV embed URL: {}", url);
        url
    }

    pub async fn get_movie_streams(&self, tmdb_id: i64) -> anyhow::Result<Vec<StreamSource>> {
        let options = EmbedOptions::default();
        let url = self.get_movie_embed_url(tmdb_id, &options);
        
        Ok(vec![StreamSource {
            id: url,
            name: "Vidking".to_string(),
            quality: Some("Auto".to_string()),
            language: Some("EN".to_string()),
            server: "vidking".to_string(),
        }])
    }

    pub async fn get_tv_streams(
        &self,
        tmdb_id: i64,
        season: i64,
        episode: i64,
    ) -> anyhow::Result<Vec<StreamSource>> {
        let options = EmbedOptions::default();
        let url = self.get_tv_embed_url(tmdb_id, season, episode, &options);
        
        Ok(vec![StreamSource {
            id: url,
            name: "Vidking".to_string(),
            quality: Some("Auto".to_string()),
            language: Some("EN".to_string()),
            server: "vidking".to_string(),
        }])
    }
}

#[derive(Debug, Clone)]
pub struct EmbedOptions {
    pub color: Option<String>,
    pub auto_play: bool,
    pub next_episode: bool,
    pub episode_selector: bool,
    pub progress: Option<i64>,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            color: Some("e50914".to_string()), // Netflix red
            auto_play: true,
            next_episode: true,
            episode_selector: true,
            progress: None,
        }
    }
}

impl EmbedOptions {
    pub fn to_query_string(&self) -> String {
        let mut params = vec![];
        
        if let Some(color) = &self.color {
            params.push(format!("color={}", color));
        }
        
        if self.auto_play {
            params.push("autoPlay=true".to_string());
        }
        
        if self.next_episode {
            params.push("nextEpisode=true".to_string());
        }
        
        if self.episode_selector {
            params.push("episodeSelector=true".to_string());
        }
        
        if let Some(progress) = self.progress {
            params.push(format!("progress={}", progress));
        }
        
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamSource {
    pub id: String,
    pub name: String,
    pub quality: Option<String>,
    pub language: Option<String>,
    pub server: String,
}