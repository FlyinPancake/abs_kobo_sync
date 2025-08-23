// empty

use serde::Deserialize;
use uuid::Uuid;

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
        let base_url_str = base_url.into();
        tracing::debug!(base_url = %base_url_str, "creating AbsClient");
        Ok(AbsClient {
            base_url: base_url_str.trim_end_matches('/').to_string(),
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_status(&self) -> anyhow::Result<StatusResponse> {
        let url = self.url("/status");
        tracing::debug!(%url, "GET status");
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
    #[tracing::instrument(level = "debug", skip(self))]
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
        tracing::debug!(%url, expanded, include = include.unwrap_or(""), "GET item");
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
        item_id: &Uuid,
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_libraries(&self) -> anyhow::Result<LibrariesResponse> {
        let url = self.url("/api/libraries");
        tracing::debug!(%url, "GET libraries");
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_library_series(
        &self,
        lib_id: &str,
        limit: i64,
        page: Option<i64>,
        filter: Option<&str>,
    ) -> anyhow::Result<LibrarySeriesResponse> {
        let url = self.url(&format!("/api/libraries/{}/series", lib_id));
        tracing::debug!(%url, %lib_id, %limit, page = page.unwrap_or(0), filter = filter.unwrap_or("") , "GET library series");
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

    /// GET /api/libraries/{lib_id}/items
    /// Common useful params: limit, page, include (e.g. "media,media.metadata"), filter
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_library_items(
        &self,
        lib_id: &Uuid,
        limit: i64,
        page: Option<i64>,
        include: Option<&str>,
        filter: Option<&str>,
    ) -> anyhow::Result<LibraryItemsResponse> {
        let url = self.url(&format!("/api/libraries/{}/items", lib_id));
        tracing::debug!(%url, %lib_id, %limit, page = page.unwrap_or(0), include = include.unwrap_or("") , filter = filter.unwrap_or("") , "GET library items");
        let req = self.client.get(&url);
        let req = if let Some((k, v)) = self.auth_header() {
            req.header(&k, &v)
        } else {
            req
        };
        // Build query parameters, keeping things resilient
        let mut q: Vec<(String, String)> = vec![
            ("limit".into(), limit.to_string()),
            ("page".into(), page.unwrap_or(0).to_string()),
        ];
        if let Some(inc) = include {
            q.push(("include".into(), inc.to_string()));
        }
        if let Some(f) = filter {
            q.push(("filter".into(), f.to_string()));
        }
        let req = req.query(&q);

        let resp = req.send().await?;
        let status = resp.error_for_status()?;
        let body = status.text().await?;
        match serde_json::from_str::<LibraryItemsResponse>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(e) => {
                let snippet_len = body.len().min(2000);
                let snippet = &body[..snippet_len];
                tracing::error!(error = %e, body_snippet = %snippet, "failed to parse LibraryItemsResponse");
                Err(e.into())
            }
        }
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
    pub id: Uuid,
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

// ============ Library Items (folders/files) ============

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemsResponse {
    pub results: Vec<LibraryItem>,
    pub total: i64,
    pub limit: i64,
    pub page: i64,
    pub sort_desc: bool,
    pub media_type: LibraryMediaType,
    pub minified: bool,
    pub collapseseries: bool,
    pub include: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LibraryMediaType {
    Book,
    Podcast,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: Uuid,
    pub ino: String,
    pub old_library_item_id: Option<String>,
    pub library_id: String,
    pub folder_id: String,
    pub path: String,
    pub rel_path: String,
    pub is_file: bool,
    /// The time when the library item was last modified on disk
    pub mtime_ms: i64,
    /// The time when the library item status was changed on disk
    pub ctime_ms: i64,
    /// The time when the library item was created on disk
    pub birthtime_ms: i64,
    /// The time when the library item was added to the library
    pub added_at: i64,
    /// The time when the library item was last updated (Read Only)
    pub updated_at: i64,
    pub is_missing: bool,
    pub is_invalid: bool,
    pub media_type: String,
    pub media: Media,
    pub num_files: i64,
    pub size: i64,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: String,
    pub metadata: BookMetadata,
    pub cover_path: Option<String>,
    pub tags: Vec<String>,
    pub num_tracks: i64,
    pub num_audio_files: i64,
    pub num_chapters: i64,
    pub duration: f64,
    pub size: i64,
    pub ebook_format: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub title_ignore_prefix: Option<String>,
    pub author_name: Option<String>,
    pub author_name_lf: Option<String>,

    pub narrator_name: Option<String>,
    pub series_name: Option<String>,
    pub genres: Vec<String>,
    #[serde(
        deserialize_with = "crate::abs_client::de::opt_i64_from_str_or_num",
        default
    )]
    pub published_year: Option<i64>,
    pub published_date: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<bool>,
    pub abridged: Option<bool>,
}

/// Internal serde helpers
pub mod de {
    use serde::{Deserialize, Deserializer};

    /// Accept Option<i64> from either a number or a string like "2011"; null/"" -> None.
    pub fn opt_i64_from_str_or_num<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum NumOrStr<'a> {
            Num(i64),
            Str(&'a str),
        }

        let val: Option<NumOrStr> = Option::deserialize(deserializer)?;
        Ok(match val {
            None => None,
            Some(NumOrStr::Num(n)) => Some(n),
            Some(NumOrStr::Str(s)) => s.trim().parse::<i64>().ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cover_url_basic() {
        let c = AbsClient::new("http://localhost:8080/audiobookshelf").unwrap();
        let url = c.cover_url(
            &Uuid::parse_str("22809dbe-3137-4879-831e-d64a6f29b005").unwrap(),
            Some((600, 800)),
            Some("jpeg"),
            false,
        );
        assert_eq!(
            url,
            "http://localhost:8080/audiobookshelf/api/items/22809dbe-3137-4879-831e-d64a6f29b005/cover?width=600&height=800&format=jpeg"
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
                        { "id": "22809dbe-3137-4879-831e-d64a6f29b005", "name": "A", "folders": [{ "id": "f1", "fullPath": "/a", "libraryId": "1", "addedAt": 1 }], "displayOrder": 1, "icon": "database", "mediaType": "book", "provider": "audible", "settings": {"coverAspectRatio":1}, "lastScan": 123, "lastScanVersion": "2.27.0", "createdAt": 1, "lastUpdate": 2 },
                        { "id": "b8df8f4c-5f93-4a10-812b-84ec4cee4389", "name": "B", "folders": [{ "id": "f2", "fullPath": "/b", "libraryId": "2", "addedAt": 2 }], "displayOrder": 2, "icon": "book-1", "mediaType": "book", "provider": "custom", "settings": {}, "lastScan": 456, "lastScanVersion": "2.25.1", "createdAt": 2, "lastUpdate": 3 },
                        { "id": "33ed2665-4521-4a70-93f1-f49b29e39bfe", "name": "C", "folders": [{ "id": "f3", "fullPath": "/c", "libraryId": "3", "addedAt": 3 }], "displayOrder": 3, "icon": "microphone-1", "mediaType": "podcast", "provider": "itunes", "settings": {}, "lastScan": null, "lastScanVersion": null, "createdAt": 3, "lastUpdate": 3 }
                    ]
                }
                "#;

        let libs: LibrariesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(libs.libraries.len(), 3);
        assert_eq!(
            libs.libraries[0].id,
            Uuid::parse_str("22809dbe-3137-4879-831e-d64a6f29b005").unwrap()
        );
        assert_eq!(libs.libraries[0].folders[0].full_path, "/a");
        assert_eq!(libs.libraries[2].media_type.as_deref(), Some("podcast"));
    }

    #[test]
    fn library_items_deserialize_example() {
        let json = r#"{
    "results": [
        {
            "id": "075ebcee-d657-4b01-a96d-b94fadb1898c",
            "ino": "552891213",
            "oldLibraryItemId": null,
            "libraryId": "55b8b4f3-2ec7-460b-8178-e02b8b619c03",
            "folderId": "381d3393-0028-41fc-95b0-e3a1afb03eec",
            "path": "/books/Wizards of The Coast",
            "relPath": "Wizards of The Coast",
            "isFile": false,
            "mtimeMs": 1738971721697,
            "ctimeMs": 1738978324038,
            "birthtimeMs": 1699116518568,
            "addedAt": 1703767976342,
            "updatedAt": 1747214658742,
            "isMissing": false,
            "isInvalid": false,
            "mediaType": "book",
            "media": {
                "id": "8f7a211c-767a-40bd-9e96-659a5c5fb6c0",
                "metadata": {
                    "title": "Player's Handbook",
                    "titleIgnorePrefix": "Player's Handbook",
                    "subtitle": null,
                    "authorName": "Richard Baker, Jeremy Crawford, Bruce R. Cordell, James Wyatt, Robert J. Schwalb",
                    "authorNameLF": "Baker, Richard, Crawford, Jeremy, Cordell, Bruce R., Wyatt, James, Schwalb, Robert J.",
                    "narratorName": "",
                    "seriesName": "",
                    "genres": [],
                    "publishedYear": null,
                    "publishedDate": null,
                    "publisher": null,
                    "description": null,
                    "isbn": null,
                    "asin": null,
                    "language": "English",
                    "explicit": false,
                    "abridged": false
                },
                "coverPath": "/books/Wizards of The Coast/cover.jpg",
                "tags": [],
                "numTracks": 0,
                "numAudioFiles": 0,
                "numChapters": 0,
                "duration": 0,
                "size": 83430656,
                "ebookFormat": "pdf"
            },
            "numFiles": 3,
            "size": 150299384
        }
    ],
    "total": 136,
    "limit": 1,
    "page": 0,
    "sortDesc": false,
    "mediaType": "book",
    "minified": false,
    "collapseseries": false,
    "include": "",
    "offset": 0
}"#;

        let parsed: LibraryItemsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.total, 136);
        assert_eq!(parsed.limit, 1);
        assert_eq!(parsed.results.len(), 1);
        let item = &parsed.results[0];
        assert_eq!(item.is_file, false);
        assert_eq!(item.media_type, "book");
        assert_eq!(item.media.ebook_format.as_deref(), Some("pdf"));
        let title = item.media.metadata.title.as_deref();
        assert_eq!(title, Some("Player's Handbook"));
    }
}
