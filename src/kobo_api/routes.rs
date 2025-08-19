use std::sync::Arc;

use base64::Engine;
use chrono::{DateTime, Utc};
use poem::{FromRequest, Request, RequestBody, http::HeaderValue};
use poem_openapi::{
    OpenApi,
    param::{Path, Query},
    payload::{Json, PlainText},
};

use super::models::{
    DeviceAuthResponseDto, EmptyOkResponseDto, InitializationResponseDto, LibraryItemsResponseDto,
    LibraryListResponse, MetadataResponseDto, NoContentResponseDto, ReadingStateGetResponseDto,
    ReadingStatePutResponseDto, SyncResponseDto, TagCreateRequestDto, TagCreateResponseDto,
    TagItemsRequestDto,
};
use super::services::{
    health::HealthService, library::LibraryService, metadata::MetadataService,
    reading::ReadingService, sync::SyncService,
};
use crate::abs_client::AbsClient;

pub struct AbsKoboApi {
    pub client: Arc<AbsClient>,
}

const KOBO_STOREAPI_URL: &str = "https://storeapi.kobo.com";

#[OpenApi]
impl AbsKoboApi {
    // /test endpoint
    #[oai(path = "/test", method = "get")]
    #[tracing::instrument(level = "debug", skip(self))]
    async fn test(&self) -> PlainText<String> {
        PlainText("Hello, world!".to_string())
    }

    // Example endpoint that uses the injected ABS client
    #[oai(path = "/status", method = "get")]
    #[tracing::instrument(level = "debug", skip(self))]
    async fn status(&self) -> PlainText<String> {
        tracing::debug!("handling /status");
        HealthService::new(&self.client).status_text().await
    }

    #[oai(path = "/v1/libraries", method = "get")]
    #[tracing::instrument(level = "debug", skip(self))]
    async fn list_libraries(&self) -> LibraryListResponse {
        LibraryService::new(&self.client).list_libraries().await
    }

    // New: list items in a library
    #[oai(path = "/v1/libraries/:library_id/items", method = "get")]
    #[tracing::instrument(level = "debug", skip(self, library_id, limit, page, include, filter))]
    async fn list_library_items(
        &self,
        library_id: Path<String>,
        /// Max items per page (default 50)
        Query(limit): Query<Option<i64>>,
        /// Page number starting at 0
        Query(page): Query<Option<i64>>,
        /// ABS include param, e.g. "media,media.metadata"
        Query(include): Query<Option<String>>,
        /// Filter string passed to ABS
        Query(filter): Query<Option<String>>,
    ) -> LibraryItemsResponseDto {
        let library_id = library_id.0;
        let limit = limit.unwrap_or(50);
        // Ensure we fetch media + metadata by default for meaningful titles
        let include_ref = include.as_deref();
        let filter_ref = filter.as_deref();
        tracing::debug!(library_id=%library_id, limit, page = page.unwrap_or(0), include = include_ref.unwrap_or(""), filter = filter_ref.unwrap_or(""), "handling list_library_items");

        LibraryService::new(&self.client)
            .list_library_items(&library_id, limit, page, include_ref, filter_ref)
            .await
    }

    // ===== Kobo sync endpoints =====

    /// Incremental sync of the user's data
    #[oai(path = "/kobo/:auth_token/v1/library/sync", method = "get")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, kobo_sync_token))]
    async fn kobo_sync(
        &self,
        Path(auth_token): Path<String>,
        kobo_sync_token: KoboSyncToken,
    ) -> SyncResponseDto {
        SyncService::new(&self.client)
            .sync(&auth_token, kobo_sync_token)
            .await
    }

    /// Metadata for a specific book (array with single object)
    #[oai(
        path = "/kobo/:auth_token/v1/library/:book_uuid/metadata",
        method = "get"
    )]
    #[tracing::instrument(level = "debug", skip(self, auth_token, book_uuid))]
    async fn book_metadata(
        &self,
        auth_token: Path<String>,
        book_uuid: Path<String>,
    ) -> MetadataResponseDto {
        let _ = auth_token;
        MetadataService::new(&self.client)
            .get_metadata(&book_uuid.0)
            .await
    }

    /// Get reading state for a specific book (array with single object)
    #[oai(path = "/kobo/:auth_token/v1/library/:book_uuid/state", method = "get")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, book_uuid))]
    async fn get_reading_state(
        &self,
        auth_token: Path<String>,
        book_uuid: Path<String>,
    ) -> ReadingStateGetResponseDto {
        let _ = auth_token;
        ReadingService::new(&self.client)
            .get_state(&book_uuid.0)
            .await
    }

    /// Update reading state for a specific book
    #[oai(path = "/kobo/:auth_token/v1/library/:book_uuid/state", method = "put")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, book_uuid, body))]
    async fn put_reading_state(
        &self,
        auth_token: Path<String>,
        book_uuid: Path<String>,
        body: poem_openapi::payload::Json<serde_json::Value>,
    ) -> ReadingStatePutResponseDto {
        let _ = auth_token;
        ReadingService::new(&self.client)
            .update_state(&book_uuid.0, body.0)
            .await
    }

    /// Create shelf (tag)
    #[oai(path = "/kobo/:auth_token/v1/library/tags", method = "post")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, body))]
    async fn create_tag(
        &self,
        auth_token: Path<String>,
        body: poem_openapi::payload::Json<TagCreateRequestDto>,
    ) -> TagCreateResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client).create_tag(body.0).await
    }

    /// Rename shelf (tag)
    #[oai(path = "/kobo/:auth_token/v1/library/tags/:tag_id", method = "put")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, tag_id, body))]
    async fn rename_tag(
        &self,
        auth_token: Path<String>,
        tag_id: Path<String>,
        body: poem_openapi::payload::Json<serde_json::Value>,
    ) -> EmptyOkResponseDto {
        let _ = auth_token;
        let name = body
            .0
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        SyncService::new(&self.client)
            .rename_tag(&tag_id.0, &name)
            .await
    }

    /// Delete shelf (tag)
    #[oai(path = "/kobo/:auth_token/v1/library/tags/:tag_id", method = "delete")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, tag_id))]
    async fn delete_tag(
        &self,
        auth_token: Path<String>,
        tag_id: Path<String>,
    ) -> EmptyOkResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client).delete_tag(&tag_id.0).await
    }

    /// Add items to shelf
    #[oai(
        path = "/kobo/:auth_token/v1/library/tags/:tag_id/items",
        method = "post"
    )]
    #[tracing::instrument(level = "debug", skip(self, auth_token, tag_id, body))]
    async fn add_tag_items(
        &self,
        auth_token: Path<String>,
        tag_id: Path<String>,
        body: poem_openapi::payload::Json<TagItemsRequestDto>,
    ) -> EmptyOkResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client)
            .add_tag_items(&tag_id.0, body.0.items)
            .await
    }

    /// Remove items from shelf
    #[oai(
        path = "/kobo/:auth_token/v1/library/tags/:tag_id/items/delete",
        method = "post"
    )]
    #[tracing::instrument(level = "debug", skip(self, auth_token, tag_id, body))]
    async fn remove_tag_items(
        &self,
        auth_token: Path<String>,
        tag_id: Path<String>,
        body: poem_openapi::payload::Json<TagItemsRequestDto>,
    ) -> EmptyOkResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client)
            .remove_tag_items(&tag_id.0, body.0.items)
            .await
    }

    /// Archive a book (device delete)
    #[oai(path = "/kobo/:auth_token/v1/library/:book_uuid", method = "delete")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, book_uuid))]
    async fn archive_book(
        &self,
        auth_token: Path<String>,
        book_uuid: Path<String>,
    ) -> NoContentResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client).archive(&book_uuid.0).await
    }

    /// Initialization resources
    #[oai(path = "/kobo/:auth_token/v1/initialization", method = "get")]
    #[tracing::instrument(level = "debug", skip(self, auth_token))]
    async fn initialization(&self, auth_token: Path<String>) -> InitializationResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client).initialization().await
    }

    /// Device auth stub
    #[oai(path = "/kobo/:auth_token/v1/auth/device", method = "post")]
    #[tracing::instrument(level = "debug", skip(self, auth_token, body))]
    async fn auth_device(
        &self,
        auth_token: Path<String>,
        Json(body): Json<serde_json::Value>,
    ) -> DeviceAuthResponseDto {
        let _ = auth_token;
        SyncService::new(&self.client).auth_device(body).await
    }
}

#[derive(Debug, Clone)]
pub enum KoboSyncToken {
    NoToken,
    OnlyRawToken {
        raw_kobo_store_token: String,
    },
    FullToken {
        raw_kobo_store_token: String,
        details: KoboFullTokenDetails,
    },
}

#[derive(Debug, Clone)]
pub struct KoboFullTokenDetails {
    pub books_last_modified: Option<DateTime<Utc>>,
    pub books_last_created: Option<DateTime<Utc>>,
    pub archive_last_modified: Option<DateTime<Utc>>,
    pub reading_state_last_modified: Option<DateTime<Utc>>,
    pub tags_last_modified: Option<DateTime<Utc>>,
}

impl KoboSyncToken {
    const HEADER_NAME: &'static str = "x-kobo-synctoken";
}

impl<'a> FromRequest<'a> for KoboSyncToken {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let token = match req.headers().get(KoboSyncToken::HEADER_NAME) {
            Some(t) => t.to_str().map_err(|_| {
                poem::Error::from_string(
                    "Invalid Kobo sync token format",
                    poem::http::StatusCode::BAD_REQUEST,
                )
            })?,
            None => return Ok(KoboSyncToken::NoToken),
        };

        // On the first sync from a Kobo device, we may receive the SyncToken
        // from the official Kobo store. Without digging too deep into it, that
        // token is of the form [b64encoded blob].[b64encoded blob 2]
        if token.contains(".") {
            return Ok(KoboSyncToken::OnlyRawToken {
                raw_kobo_store_token: token.to_string(),
            });
        }

        // At this point we can assume that the token is a single json object encoded as base64
        let json = base64::prelude::BASE64_STANDARD
            .decode(token)
            .map_err(|_| {
                poem::Error::from_string(
                    "Invalid Kobo sync token format",
                    poem::http::StatusCode::BAD_REQUEST,
                )
            })?;

        let values = serde_json::from_slice::<serde_json::Value>(&json).map_err(|_| {
            poem::Error::from_string(
                "Invalid Kobo sync token JSON format",
                poem::http::StatusCode::BAD_REQUEST,
            )
        })?;

        let raw_kobo_store_token = match values
            .get("raw_kobo_store_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            Some(raw_kobo_store_token) => raw_kobo_store_token,
            None => {
                return Ok(KoboSyncToken::NoToken);
            }
        };

        let books_last_modified = values
            .get("books_last_modified")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let books_last_created = values
            .get("books_last_created")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let archive_last_modified = values
            .get("archive_last_modified")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let reading_state_last_modified = values
            .get("reading_state_last_modified")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let tags_last_modified = values
            .get("tags_last_modified")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(KoboSyncToken::FullToken {
            raw_kobo_store_token,
            details: KoboFullTokenDetails {
                books_last_modified,
                books_last_created,
                archive_last_modified,
                reading_state_last_modified,
                tags_last_modified,
            },
        })
    }
}
