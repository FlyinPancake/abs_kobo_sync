use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use poem::web::headers::Date;
use poem_openapi::{Enum, Object, Union, payload::Json};
use serde::de;
use serde_json::json;
use uuid::Uuid;

use crate::{
    abs_client::AbsClient,
    kobo_api::{
        models::{
            BookFormatDto, DeviceAuthResponseDto, EmptyOkResponseDto, InitializationResponseDto,
            NoContentResponseDto, SyncResponseDto, TagCreateRequestDto, TagCreateResponseDto,
            TagItemDto,
        },
        routes::{KoboFullTokenDetails, KoboSyncToken},
    },
};
// no_std: poem-openapi will serialize headers

pub struct SyncService<'a> {
    pub abs_client: &'a AbsClient,
}

impl<'a> SyncService<'a> {
    pub fn new(abs_client: &'a AbsClient) -> Self {
        Self { abs_client }
    }

    // TODO: replace with actual urls
    #[tracing::instrument(level = "debug", skip(self, format))]
    fn get_download_url_for_book(&self, book_id: &Uuid, format: &BookFormatDto) -> String {
        format!(
            "https://example.com/download/{}/{}",
            book_id,
            format.to_string()
        )
    }

    const SYNC_ITEM_LIMIT: i64 = 100;

    #[tracing::instrument(level = "debug", skip(self, auth_token))]
    fn collect_sync_items(&self, auth_token: &str) -> Vec<SyncResult> {
        // TODO: implement actual sync item collection logic

        todo!("Implement sync item collection")
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn sync(&self, auth_token: &str, kobo_sync_token: KoboSyncToken) -> SyncResponseDto {
        // Minimal stub: no changes; return empty list with a dummy sync token
        let _ = auth_token;

        tracing::info!("Kobo Sync Token Received");
        tracing::info!(?kobo_sync_token, "Kobo Sync Token Details");
        tracing::info!(
            "Download link format: {}",
            // TODO: replace with actual implementation
            "https://example.com/download/{book_id}/{format}"
        );

        // Check kobo token. If No token, return with 400, if only raw token was provided set local timestamps to unix epoch, else use the values from the token
        let token_details = match kobo_sync_token {
            KoboSyncToken::NoToken => {
                return SyncResponseDto::Unauthorized(Json(crate::kobo_api::models::ErrorDto {
                    message: "Kobo Sync Token is required".to_string(),
                }));
            }
            KoboSyncToken::OnlyRawToken { .. } => KoboFullTokenDetails {
                books_last_modified: None,
                books_last_created: None,
                archive_last_modified: None,
                reading_state_last_modified: None,
                tags_last_modified: None,
            },
            KoboSyncToken::FullToken { details, .. } => details,
        };

        // TODO: check if the user has ever synced books for this kobo, and if not, set the
        let KoboFullTokenDetails {
            books_last_modified,
            books_last_created,
            archive_last_modified,
            reading_state_last_modified,
            tags_last_modified,
        } = if false {
            KoboFullTokenDetails {
                books_last_modified: None,
                books_last_created: None,
                reading_state_last_modified: None,
                archive_last_modified: token_details.archive_last_modified,
                tags_last_modified: token_details.tags_last_modified,
            }
        } else {
            token_details
        };

        let archive_last_modified: Option<DateTime<Utc>> = None;

        let sync_results = self.collect_sync_items(auth_token);

        let token = URL_SAFE_NO_PAD.encode("initial");
        SyncResponseDto::Ok(Json(vec![]), None, None, None)
    }

    #[tracing::instrument(level = "debug", skip(self, req))]
    pub async fn create_tag(&self, req: TagCreateRequestDto) -> TagCreateResponseDto {
        if req.name.trim().is_empty() {
            return TagCreateResponseDto::BadRequest(Json(crate::kobo_api::models::ErrorDto {
                message: "Name is required".to_string(),
            }));
        }
        let id = Uuid::new_v4().to_string();
        TagCreateResponseDto::Created(Json(id))
    }

    #[tracing::instrument(level = "debug", skip(self, _tag_id, _name))]
    pub async fn rename_tag(&self, _tag_id: &str, _name: &str) -> EmptyOkResponseDto {
        EmptyOkResponseDto::Ok
    }

    #[tracing::instrument(level = "debug", skip(self, _tag_id))]
    pub async fn delete_tag(&self, _tag_id: &str) -> EmptyOkResponseDto {
        EmptyOkResponseDto::Ok
    }

    #[tracing::instrument(level = "debug", skip(self, _tag_id, _items))]
    pub async fn add_tag_items(
        &self,
        _tag_id: &str,
        _items: Vec<TagItemDto>,
    ) -> EmptyOkResponseDto {
        EmptyOkResponseDto::Ok
    }

    #[tracing::instrument(level = "debug", skip(self, _tag_id, _items))]
    pub async fn remove_tag_items(
        &self,
        _tag_id: &str,
        _items: Vec<TagItemDto>,
    ) -> EmptyOkResponseDto {
        EmptyOkResponseDto::Ok
    }

    #[tracing::instrument(level = "debug", skip(self, _book_uuid))]
    pub async fn archive(&self, _book_uuid: &str) -> NoContentResponseDto {
        NoContentResponseDto::NoContent
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn initialization(&self) -> InitializationResponseDto {
        // Minimal resources structure used by devices. Can be extended later.
        let resources = json!({
            "Resources": {
                // Keep keys matching device expectations (UpperCamelCase vs lower per spec)
                "image_host": "",
                "image_url_template": "/kobo/{authToken}/v1/books/{ImageId}/thumbnail/{Width}/{Height}/false/image.jpg",
                "image_url_quality_template": "/kobo/{authToken}/v1/books/{ImageId}/thumbnail/{Width}/{Height}/{Quality}/{IsGreyscale}/image.jpg"
            }
        });
        InitializationResponseDto::Ok(Json(resources))
    }

    #[tracing::instrument(level = "debug", skip(self, body))]
    pub async fn auth_device(&self, body: serde_json::Value) -> DeviceAuthResponseDto {
        let user_key = body.get("UserKey").cloned().unwrap_or(json!(""));
        let resp = json!({
            "AccessToken": Uuid::new_v4().to_string(),
            "RefreshToken": Uuid::new_v4().to_string(),
            "TrackingId": Uuid::new_v4().to_string(),
            "ExpiresIn": 3600,
            "TokenType": "Bearer",
            "UserKey": user_key
        });
        DeviceAuthResponseDto::Ok(Json(resp))
    }
}

// TODO: replace this struct with actual implementation
/// Represents one book's sync result.
#[derive(Debug, Clone)]
struct SyncResult {
    book_id: Uuid,
    download_url: String,
    format: BookFormatDto,
    last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct BookEntitlement {
    accessibility: String, // TODO: find out what this value actually means
    active_period: ActivePeriod,
    created: DateTime<Utc>,
    cross_revision_id: Uuid,
    is_removed: bool,
    is_hidden_from_archive: bool,
    is_locked: bool,
    last_modified: DateTime<Utc>,
    origin_category: String, // TODO: find all the valid options
    revision_id: Uuid,
    status: String, // TODO: find all the valid options
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct ActivePeriod {
    from: DateTime<Utc>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct BookMetadata {
    categories: Vec<Uuid>,
    cover_image_id: Uuid,
    cross_revision_id: Uuid,
    current_display_price: ContentDisplayPrice,
    current_love_display_price: CurrentLoveDisplayPrice,
    description: Option<String>,
    download_urls: Vec<String>,
    entitlement_id: Uuid,
    external_ids: Vec<Uuid>,
    genre: Uuid,
    is_eligible_for_kobo_love: bool,
    is_internet_archive: bool,
    is_pre_order: bool,
    is_social_enabled: bool,
    language: String,
    phonetic_pronunciations: PhoneticPronounciations,
    publication_date: DateTime<Utc>,
    revision_id: Uuid,
    title: String,
    work_id: Uuid,
    contributors: Vec<String>,
    contributor_roles: Vec<KoboSyncedContributorRole>,
    series: KoboSyncedSeries,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct ContentDisplayPrice {
    currency_code: String,
    total_amount: f64,
}

impl Default for ContentDisplayPrice {
    fn default() -> Self {
        Self {
            currency_code: "USD".to_string(),
            total_amount: 0.0,
        }
    }
}

#[derive(Debug, Clone, Object, Default)]
#[oai(rename_all = "PascalCase")]
pub struct CurrentLoveDisplayPrice {
    total_amount: f64,
}

#[derive(Debug, Clone, Object)]
pub struct PhoneticPronounciations {}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboPublisher {
    imprint: String,
    name: Option<String>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedContributorRole {
    name: String,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedSeries {
    name: String,
    number: f64,
    number_float: f64,
    id: Uuid,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedReadingState {
    entitlement_id: Uuid,
    created: DateTime<Utc>,
    last_modified: DateTime<Utc>,
    priority_timestamp: DateTime<Utc>,
    status_info: KoboSyncedStatusInfo,
    statistics: KoboSyncedStatistics,
    current_bookmark: KoboCurrentBookmark,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedStatusInfo {
    last_modified: DateTime<Utc>,
    status: KoboSyncedStatus,
    times_started_read: f64,
    last_time_started_read: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Enum)]
#[oai(rename_all = "PascalCase")]
pub enum KoboSyncedStatus {
    ReadyToRead,
    Finished,
    Reading,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedStatistics {
    last_modified: DateTime<Utc>,
    spent_reading_minutes: Option<f64>,
    remaining_reading_minutes: Option<f64>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboCurrentBookmark {
    last_modified: DateTime<Utc>,
    progress_percent: Option<f64>,
    content_source_progress_percent: Option<f64>,
    location: Option<KoboCurrentBookmarkLocation>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboCurrentBookmarkLocation {
    value: String,
    #[oai(rename = "Type")]
    _type: String,
    source: String,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct KoboSyncedBook {
    book_entitlement: BookEntitlement,
    book_metadata: BookMetadata,
    reading_state: Option<KoboSyncedReadingState>,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct NewEntitlement {
    new_entitlement: KoboSyncedBook,
}

#[derive(Debug, Clone, Object)]
#[oai(rename_all = "PascalCase")]
pub struct ChangedEntitlement {
    changed_entitlement: KoboSyncedBook,
}

#[derive(Debug, Clone, Union)]

pub enum KoboSyncEntitlement {
    NewEntitlement(NewEntitlement),
    ChangedEntitlement(ChangedEntitlement),
}
