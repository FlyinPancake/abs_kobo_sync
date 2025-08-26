use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use entities::{book_sync, devices, prelude::BookSync, user};
use poem::http::HeaderMap;
use poem_openapi::payload::Json;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::json;
use uuid::Uuid;

use crate::{
    AbsKoboResult,
    abs_client::{AbsClient, LibraryItem},
    config::Config,
    kobo_api::{
        models::*,
        routes::{KoboFullTokenDetails, KoboSyncToken},
    },
};
// no_std: poem-openapi will serialize headers

pub struct SyncService<'a> {
    pub abs_client: &'a AbsClient,
    pub config: &'a Config,
    pub db: &'a DatabaseConnection,
}

static KOBO_STOREAPI_URL: &str = "https://storeapi.kobo.com";
static KOBO_IMAGEHOST_URL: &str = "https://cdn.kobo.com/book-images";

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
    fn get_download_url_for_book(&self, library_item_id: &Uuid, format: &BookFormatDto) -> String {
        format!("https://example.com/download/{}", library_item_id,)
    }

    async fn get_api_key(&self, device_id: Uuid) -> AbsKoboResult<Option<String>> {
        if let Some((_, Some(user))) = devices::Entity::find_by_id(device_id)
            .select_also(user::Entity)
            .one(self.db)
            .await?
        {
            Ok(Some(user.abs_api_key))
        } else {
            Ok(None)
        }
    }

    const SYNC_ITEM_LIMIT: usize = 100;

    #[tracing::instrument(level = "debug", skip(self, auth_token, books_last_modified))]
    async fn collect_books_to_sync(
        &self,
        auth_token: Uuid,
        books_last_modified: &Option<DateTime<Utc>>,
    ) -> AbsKoboResult<Vec<(SyncType, LibraryItem)>> {
        let user_api_key = self.get_api_key(auth_token).await?;
        let user_api_key = match user_api_key {
            Some(key) => key,
            None => {
                tracing::error!("No API key found for device {}", auth_token);
                return Ok(vec![]);
            }
        };

        let books = self
            .abs_client
            .get_library_items(&self.config.library_id, 0, None, None, None, &user_api_key)
            .await?;

        // Get the last modified timestamp for books or fall back to UNIX_EPOCH
        let books_last_modified =
            books_last_modified.unwrap_or_else(|| DateTime::<Utc>::from(std::time::UNIX_EPOCH));

        // Build a hashmap from the already synced book IDs
        let already_synced_ids: HashMap<Uuid, book_sync::Model> = BookSync::find()
            .filter(book_sync::Column::DeviceId.eq(auth_token))
            .all(self.db)
            .await?
            .into_iter()
            .map(|record| {
                (
                    Uuid::parse_str(&record.abs_item_id).expect("Invalid UUID from DB"),
                    record,
                )
            })
            .collect();

        let book_list = books.results.into_iter().filter_map(|item| {
            // Filter for recently added books
            if item.media.ebook_format == Some("epub".to_string()) {
                return None;
            }

            let added_date = Utc.timestamp_opt(item.added_at, 0).unwrap();
            let is_recently_added = added_date > books_last_modified;

            // Filter for recently updated books
            let updated_date = Utc.timestamp_opt(item.updated_at, 0).unwrap();
            let is_recently_updated = updated_date > books_last_modified;

            // Filter books for updates after last sync
            let current_version_synced =
                if let Some(existing_sync_item) = already_synced_ids.get(&item.id) {
                    updated_date <= existing_sync_item.timestamp
                } else {
                    false
                };

            if (is_recently_added || is_recently_updated) && !current_version_synced {
                if already_synced_ids.contains_key(&item.id) {
                    Some((SyncType::Update, item))
                } else {
                    Some((SyncType::New, item))
                }
            } else {
                None
            }
        });

        Ok(book_list.collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn sync(
        &self,
        auth_token: Uuid,
        raw_kobo_sync_token: String,
        headers: &HeaderMap,
    ) -> SyncResponseDto {
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
            .collect_books_to_sync(auth_token, &books_last_modified)
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

        tracing::info!("Collected {} books to sync", sync_results.len());
        let book_count = sync_results.len();

        // limit sync items
        let sync_results: Vec<_> = sync_results
            .into_iter()
            .take(Self::SYNC_ITEM_LIMIT)
            .collect();

        let mut entitlements = Vec::new();
        for (sync_type, result) in &sync_results {
            let download_urls =
                vec![self.get_download_url_for_book(&result.id, &BookFormatDto::Kepub)];

            let book_metadata =
                match BookMetadata::try_from_library_item(result.clone(), download_urls) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to create book metadata");
                        continue;
                    }
                };

            let book_entitlement = BookEntitlement::from_library_item(result);

            let reading_state = None;

            let book = KoboSyncedBook {
                book_entitlement,
                book_metadata,
                reading_state,
            };
            entitlements.push((sync_type, book));

            // Remove previous sync entries for this book
            book_sync::Entity::delete_many()
                .filter(book_sync::Column::DeviceId.eq(auth_token))
                .filter(book_sync::Column::AbsItemId.eq(result.id.to_string()))
                .exec(self.db)
                .await
                .ok();

            // Insert new sync entry for this book
            book_sync::Entity::insert(book_sync::ActiveModel {
                id: Set(Uuid::now_v7()),
                device_id: Set(auth_token),
                abs_item_id: Set(result.id.to_string()),
                timestamp: Set(Utc::now()),
            })
            .exec(self.db)
            .await
            .ok();
        }

        let entitlements = entitlements
            .into_iter()
            .map(|(sync_type, entitlement)| match sync_type {
                SyncType::New => KoboSyncEntitlement::NewEntitlement(NewEntitlement {
                    new_entitlement: entitlement,
                }),
                SyncType::Update => KoboSyncEntitlement::ChangedEntitlement(ChangedEntitlement {
                    changed_entitlement: entitlement,
                }),
            })
            .collect::<Vec<_>>();

        let kobo_sync_token = KoboFullTokenDetails {
            books_last_modified,
            books_last_created,
            archive_last_modified,
            reading_state_last_modified,
            tags_last_modified,
        };

        let rq_client = reqwest::Client::new();
        let req = rq_client
            .get(format!("{}/v1/library/sync", KOBO_STOREAPI_URL))
            .headers(headers.clone())
            .header("Host", "")
            .header(KoboSyncToken::HEADER_NAME, kobo_sync_token.to_raw_token());

        let resp = match req.send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!(error = %e, "Failed to send sync request");
                return SyncResponseDto::BadGateway(Json(crate::kobo_api::models::ErrorDto {
                    message: format!("Failed to send sync request: {}", e),
                }));
            }
        };

        let kobo_storeapi_headers = resp.headers().clone();
        let kobo_storeapi_raw_token = kobo_storeapi_headers
            .get(KoboSyncToken::HEADER_NAME)
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or("".to_string());
        let x_kobo_sync = kobo_storeapi_headers
            .get("x-kobo-sync")
            .map(|v| v.to_str().unwrap_or("").to_string());
        let x_kobo_sync_mode = kobo_storeapi_headers
            .get("x-kobo-sync-mode")
            .map(|v| v.to_str().unwrap_or("").to_string());
        let x_kobo_recent_reads = kobo_storeapi_headers
            .get("x-kobo-recent-reads")
            .map(|v| v.to_str().unwrap_or("").to_string());

        let kobo_store_entitlements: Vec<KoboSyncEntitlement> = {
            let text = resp.text().await.expect("Failed to read response text");
            serde_json::from_str(&text).expect("Failed to parse response JSON")
        };

        let all_entitlements = [entitlements, kobo_store_entitlements].concat();

        let x_kobo_sync = if book_count > Self::SYNC_ITEM_LIMIT {
            Some("continue".to_string())
        } else {
            x_kobo_sync
        };

        SyncResponseDto::Ok(
            Json(all_entitlements),
            kobo_storeapi_raw_token,
            x_kobo_sync,
            x_kobo_sync_mode,
            x_kobo_recent_reads,
        )
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

/// Represents the type of sync request
enum SyncType {
    /// New book appeared
    New,
    /// Book was updated, requiring re-sync
    Update,
}
