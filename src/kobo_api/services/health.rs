use poem_openapi::payload::PlainText;

use crate::abs_client::AbsClient;

pub struct HealthService<'a> {
    pub client: &'a AbsClient,
}

impl<'a> HealthService<'a> {
    pub fn new(client: &'a AbsClient) -> Self {
        Self { client }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn status_text(&self) -> PlainText<String> {
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
