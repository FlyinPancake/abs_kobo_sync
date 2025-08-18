// empty

use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct AbsClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl AbsClient {
    /// Create a new client with the given base URL (e.g. "http://localhost:8080/audiobookshelf").
    pub fn new(base_url: impl Into<String>) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(AbsClient {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
            client,
        })
    }

    /// Return a client with the provided API key set (Bearer)
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    fn auth_header(&self) -> Option<(String, String)> {
        self.api_key
            .as_ref()
            .map(|k| ("Authorization".to_string(), format!("Bearer {}", k)))
    }

    /// GET /status (no auth required)
    pub async fn get_status(&self) -> anyhow::Result<StatusResponse> {
        let url = self.url("/status");
        let mut req = self.client.get(&url);
        if let Some((k, v)) = self.auth_header() {
            req = req.header(&k, &v);
        }
        let resp = req.send().await?;
        let status = resp.error_for_status()?;
        let body = status.text().await?;
        let parsed: StatusResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    /// GET /api/items/:id
    pub async fn get_item(
        &self,
        item_id: &str,
        expanded: bool,
        include: Option<&str>,
    ) -> anyhow::Result<ItemResponse> {
        let mut path = format!("/api/items/{}", item_id);
        let mut q = vec![];
        if expanded {
            q.push(("expanded", "1"));
        }
        if let Some(include) = include {
            q.push(("include", include));
        }
        if !q.is_empty() {
            let qs: String = q
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            path = format!("{}?{}", path, qs);
        }

        let url = self.url(&path);
        let mut req = self.client.get(&url);
        if let Some((k, v)) = self.auth_header() {
            req = req.header(&k, &v);
        }
        let resp = req.send().await?;
        let status = resp.error_for_status()?;
        let body = status.text().await?;
        let parsed: ItemResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    /// Build cover URL for an item. This returns a public URL and does not perform a request.
    /// Example: client.cover_url("ITEM_ID", Some((600, 800)), Some("jpeg"), false)
    pub fn cover_url(
        &self,
        item_id: &str,
        size: Option<(u32, u32)>,
        format: Option<&str>,
        raw: bool,
    ) -> String {
        let mut path = format!("/api/items/{}/cover", item_id);
        let mut q = vec![];
        if let Some((w, h)) = size {
            q.push(format!("width={}", w));
            q.push(format!("height={}", h));
        }
        if let Some(fmt) = format {
            q.push(format!("format={}", fmt));
        }
        if raw {
            q.push("raw=1".to_string());
        }
        if !q.is_empty() {
            path = format!("{}?{}", path, q.join("&"));
        }
        self.url(&path)
    }

    /// GET /api/libraries
    pub async fn get_libraries(&self) -> anyhow::Result<LibrariesResponse> {
        let url = self.url("/api/libraries");
        let mut req = self.client.get(&url);
        if let Some((k, v)) = self.auth_header() {
            req = req.header(&k, &v);
        }
        let resp = req.send().await?;
        let status = resp.error_for_status()?;
        let body = status.text().await?;
        let parsed: LibrariesResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    /// GET /api/libraries/{lib_id}/series
    pub async fn get_library_series(
        &self,
        lib_id: &str,
        limit: i64,
        page: Option<i64>,
        filter: Option<&str>,
    ) -> anyhow::Result<LibrarySeriesResponse> {
        let url = self.url(&format!("/api/libraries/{}/series", lib_id));
        let req = self.client.get(&url);
        let req = if let Some((k, v)) = self.auth_header() {
            req.header(&k, &v)
        } else {
            req
        };
        let req = req.query(&[
            ("limit", limit.to_string()),
            ("filter", filter.unwrap_or("").to_string()),
            ("page", page.unwrap_or(0).to_string()),
        ]);

        let resp = req.send().await?;
        let status = resp.error_for_status()?;
        let body = status.text().await?;
        let parsed: LibrarySeriesResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StatusResponse {
    pub app: Option<String>,
    #[serde(rename = "serverVersion")]
    pub server_version: Option<String>,
    #[serde(rename = "isInit")]
    pub is_init: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ItemResponse {
    pub id: String,
    pub title: Option<String>,
    // allow extra fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LibrariesResponse {
    pub libraries: Vec<Library>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub folders: Vec<LibraryFolder>,
    #[serde(rename = "displayOrder")]
    pub display_order: Option<i64>,
    pub icon: Option<String>,
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    pub provider: Option<String>,
    pub settings: Option<serde_json::Value>,
    #[serde(rename = "lastScan")]
    pub last_scan: Option<serde_json::Value>,
    #[serde(rename = "lastScanVersion")]
    pub last_scan_version: Option<Option<String>>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<i64>,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LibraryFolder {
    pub id: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(rename = "libraryId")]
    pub library_id: String,
    #[serde(rename = "addedAt")]
    pub added_at: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LibrarySeriesResponse {
    pub results: Vec<LibrarySeries>,
    pub total: i64,
    pub limit: i64,
    pub page: i64,
    #[serde(rename = "sortDesc")]
    pub sort_desc: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LibrarySeries {
    pub id: String,
    pub name: String,
    // pub books: Vec<LibraryBook>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cover_url_basic() {
        let c = AbsClient::new("http://localhost:8080/audiobookshelf").unwrap();
        let url = c.cover_url("abc123", Some((600, 800)), Some("jpeg"), false);
        assert_eq!(
            url,
            "http://localhost:8080/audiobookshelf/api/items/abc123/cover?width=600&height=800&format=jpeg"
        );
    }

    #[test]
    fn status_deserialize() {
        let json = r#"{ "app": "audiobookshelf", "serverVersion": "2.3.4", "isInit": true }"#;
        let s: StatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(s.app.unwrap(), "audiobookshelf");
        assert_eq!(s.server_version.unwrap(), "2.3.4");
        assert_eq!(s.is_init.unwrap(), true);
    }

    #[test]
    fn libraries_deserialize_example() {
        let json = r#"
                {
                    "libraries": [
                        { "id": "1", "name": "A", "folders": [{ "id": "f1", "fullPath": "/a", "libraryId": "1", "addedAt": 1 }], "displayOrder": 1, "icon": "database", "mediaType": "book", "provider": "audible", "settings": {"coverAspectRatio":1}, "lastScan": 123, "lastScanVersion": "2.27.0", "createdAt": 1, "lastUpdate": 2 },
                        { "id": "2", "name": "B", "folders": [{ "id": "f2", "fullPath": "/b", "libraryId": "2", "addedAt": 2 }], "displayOrder": 2, "icon": "book-1", "mediaType": "book", "provider": "custom", "settings": {}, "lastScan": 456, "lastScanVersion": "2.25.1", "createdAt": 2, "lastUpdate": 3 },
                        { "id": "3", "name": "C", "folders": [{ "id": "f3", "fullPath": "/c", "libraryId": "3", "addedAt": 3 }], "displayOrder": 3, "icon": "microphone-1", "mediaType": "podcast", "provider": "itunes", "settings": {}, "lastScan": null, "lastScanVersion": null, "createdAt": 3, "lastUpdate": 3 }
                    ]
                }
                "#;

        let libs: LibrariesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(libs.libraries.len(), 3);
        assert_eq!(libs.libraries[0].id, "1");
        assert_eq!(libs.libraries[0].folders[0].full_path, "/a");
        assert_eq!(libs.libraries[2].media_type.as_deref(), Some("podcast"));
    }
}
