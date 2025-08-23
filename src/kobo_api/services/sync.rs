use chrono::{DateTime, TimeZone, Utc};
use poem_openapi::{Enum, Object, Union, payload::Json};
use sea_orm::DatabaseConnection;
use serde_json::json;
use uuid::Uuid;

use crate::{
    AbsKoboResult,
    abs_client::{AbsClient, LibraryItem},
    config::Config,
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
    pub config: &'a Config,
    pub db: &'a DatabaseConnection,
}

fn timestamp_to_utc(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp, 0).unwrap()
}

impl<'a> SyncService<'a> {
    pub fn new(abs_client: &'a AbsClient, config: &'a Config, db: &'a DatabaseConnection) -> Self {
        Self {
            abs_client,
            config,
            db,
        }
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

    #[tracing::instrument(level = "debug", skip(self, _auth_token, books_last_modified))]
    async fn collect_all_books(
        &self,
        _auth_token: &str,
        books_last_modified: &Option<DateTime<Utc>>,
    ) -> AbsKoboResult<Vec<LibraryItem>> {
        let books = self
            .abs_client
            .get_library_items(&self.config.library_id, 0, None, None, None)
            .await?;

        let book_list = books.results.into_iter().filter(|item| {
            // Filter books based on last modified date
            let added_date = Utc.timestamp_opt(item.added_at, 0).unwrap();

            let is_recent = if let Some(last_modified) = books_last_modified {
                added_date > *last_modified
            } else {
                true // If no last modified date, include all books
            };
            let has_epub = item.media.ebook_format == Some("epub".to_string());
            is_recent && has_epub
        });

        let library_items: Vec<LibraryItem> = book_list.collect();

        Ok(library_items)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn sync(&self, auth_token: &str, raw_kobo_sync_token: String) -> SyncResponseDto {
        // Minimal stub: no changes; return empty list with a dummy sync token
        let _ = auth_token;
        let kobo_sync_token = match KoboSyncToken::from_request(&raw_kobo_sync_token) {
            Ok(token) => token,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse Kobo Sync Token");
                return SyncResponseDto::Forbidden(Json(crate::kobo_api::models::ErrorDto {
                    message: format!("Invalid Kobo Sync Token: {}", e),
                }));
            }
        };

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

        let sync_results = match self
            .collect_all_books(auth_token, &books_last_modified)
            .await
        {
            Ok(results) => results,
            Err(e) => {
                tracing::error!(error = %e, "Failed to collect books for sync");
                return SyncResponseDto::BadGateway(Json(crate::kobo_api::models::ErrorDto {
                    message: format!("Failed to collect books for sync: {}", e),
                }));
            }
        };

        let entitlements = sync_results.iter().map(|result| {
            let download_urls =
                vec![self.get_download_url_for_book(&result.id, &BookFormatDto::Kepub)];

            let reading_state = None;

            let book = KoboSyncedBook {
                book_entitlement: BookEntitlement {
                    accessibility: "Full".to_string(),
                    active_period: ActivePeriod { from: Utc::now() },
                    created: timestamp_to_utc(result.added_at),
                    cross_revision_id: result.id,
                    id: result.id,
                    is_removed: false,
                    is_hidden_from_archive: false,
                    is_locked: false,
                    last_modified: timestamp_to_utc(result.updated_at),
                    origin_category: "Imported".to_string(),
                    revision_id: result.id,
                    status: "Active".to_string(),
                },
                book_metadata: BookMetadata {
                    categories: vec![
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    ],
                    cover_image_id: result.id,
                    cross_revision_id: result.id,
                    current_display_price: Default::default(),
                    current_love_display_price: Default::default(),
                    description: result.media.metadata.description.clone(),
                    download_urls,
                    entitlement_id: result.id,
                    external_ids: vec![],
                    genre: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    is_eligible_for_kobo_love: false,
                    is_internet_archive: false,
                    is_pre_order: false,
                    is_social_enabled: true,
                    language: result
                        .media
                        .metadata
                        .language
                        .clone()
                        .unwrap_or("en".to_string()),
                    phonetic_pronunciations: PhoneticPronounciations {},
                    publication_date: DateTime::parse_from_rfc3339(
                        &result
                            .media
                            .metadata
                            .published_date
                            .clone()
                            .unwrap_or("1970-01-01T00:00:00Z".to_string()),
                    )
                    .unwrap()
                    .to_utc(),
                    revision_id: result.id,
                    title: result
                        .media
                        .metadata
                        .title
                        .clone()
                        .unwrap_or("Untitled".to_string()),
                    work_id: result.id,
                    contributors: result
                        .media
                        .metadata
                        .author_name
                        .clone()
                        .map(|author| author.split(',').map(|s| s.trim().to_string()).collect()),
                    contributor_roles: result.media.metadata.author_name.clone().map(|author| {
                        author
                            .split(',')
                            .map(|s| KoboSyncedContributorRole {
                                name: s.trim().to_string(),
                            })
                            .collect()
                    }),
                    series: KoboSyncedSeries {
                        name: result
                            .media
                            .metadata
                            .series_name
                            .clone()
                            .unwrap_or("".to_string()),
                        number: 1f64,
                        number_float: 1f64,
                        id: Uuid::new_v3(
                            &Uuid::NAMESPACE_DNS,
                            &result
                                .media
                                .metadata
                                .series_name
                                .clone()
                                .unwrap_or("".to_string())
                                .as_bytes(),
                        ),
                    },
                },
                reading_state,
            };
            book
        });

        let entitlements = entitlements
            .into_iter()
            .map(|entitlement| {
                KoboSyncEntitlement::NewEntitlement(NewEntitlement {
                    new_entitlement: entitlement,
                })
            })
            .collect::<Vec<_>>();

        SyncResponseDto::Ok(Json(entitlements), None, None, None)
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
    id: Uuid,
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
    contributors: Option<Vec<String>>,
    contributor_roles: Option<Vec<KoboSyncedContributorRole>>,
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
