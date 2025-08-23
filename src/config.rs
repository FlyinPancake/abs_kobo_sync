use std::path::PathBuf;

use anyhow::Context;
use uuid::Uuid;

#[derive(Debug)]
pub struct Config {
    pub abs_api_key: String,
    pub abs_base_url: String,
    pub kepubify_path: String,
    pub db_connection_string: String,
    pub library_id: Uuid,
}

const DEFAULT_KEPUBIFY_PATH: &str = "kepubify";
const DEFAULT_DB_CONNECTION_STRING: &str = "sqlite://db.sqlite?mode=rwc";

impl Config {
    pub fn load() -> Self {
        let abs_api_key = std::env::var("ABS_API_KEY").unwrap_or_default();
        let abs_base_url = std::env::var("ABS_BASE_URL").unwrap_or_default();
        let kepubify_path = std::env::var("KEPUBIFY_PATH").unwrap_or(DEFAULT_KEPUBIFY_PATH.into());
        let db_connection_string =
            std::env::var("DB_CONNECTION_STRING").unwrap_or(DEFAULT_DB_CONNECTION_STRING.into());
        let library_id = std::env::var("LIBRARY_ID").unwrap_or_default();
        Config {
            abs_api_key,
            abs_base_url,
            kepubify_path,
            db_connection_string,
            library_id: Uuid::parse_str(&library_id)
                .with_context(|| format!("Invalid LIBRARY_ID: {}", library_id))
                .unwrap(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.abs_api_key.is_empty() {
            return Err("ABS_API_KEY is missing".into());
        }
        if self.abs_base_url.is_empty() {
            return Err("ABS_BASE_URL is missing".into());
        }
        Ok(())
    }
}
