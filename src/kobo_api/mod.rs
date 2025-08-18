use std::sync::Arc;

use crate::abs_client::AbsClient;
use poem_openapi::{OpenApi, payload::PlainText};

pub struct AbsKoboApi {
    pub client: Arc<AbsClient>,
}
#[OpenApi]
impl AbsKoboApi {
    // /test endpoint
    #[oai(path = "/test", method = "get")]
    async fn test(&self) -> PlainText<String> {
        PlainText("Hello, world!".to_string())
    }

    // Example endpoint that uses the injected ABS client
    #[oai(path = "/status", method = "get")]
    async fn status(&self) -> PlainText<String> {
        match self.client.get_status().await {
            Ok(s) => PlainText(format!(
                "ABS app={} version={}",
                s.app.unwrap_or_default(),
                s.server_version.unwrap_or_default()
            )),
            Err(e) => PlainText(format!("error: {}", e)),
        }
    }
}
