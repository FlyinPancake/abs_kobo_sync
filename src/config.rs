use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub abs_api_key: String,
    pub abs_base_url: String,
    pub kepubify_path: String,
}

impl Config {
    pub fn load() -> Self {
        let abs_api_key = std::env::var("ABS_API_KEY").unwrap_or_default();
        let abs_base_url = std::env::var("ABS_BASE_URL").unwrap_or_default();
        let kepubify_path = std::env::var("KEPUBIFY_PATH").unwrap_or("kepubify".into());
        Config {
            abs_api_key,
            abs_base_url,
            kepubify_path,
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
