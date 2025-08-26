use chrono::{DateTime, TimeZone as _, Utc};
use poem_openapi::{Enum, Object, Union};
use serde::Deserialize;
use uuid::Uuid;

use crate::abs_client::LibraryItem;

fn timestamp_to_utc(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp, 0).unwrap()
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct BookEntitlement {
    pub accessibility: BookAccessibility,
    pub active_period: ActivePeriod,
    pub created: DateTime<Utc>,
    pub cross_revision_id: Uuid,
    pub id: Uuid,
    pub is_removed: bool,
    pub is_hidden_from_archive: bool,
    pub is_locked: bool,
    pub last_modified: DateTime<Utc>,
    pub origin_category: BookOriginCategory,
    pub revision_id: Uuid,
    pub status: BookStatus,
}

impl BookEntitlement {
    pub fn from_library_item(item: &LibraryItem) -> Self {
        Self {
            accessibility: Default::default(),
            active_period: Default::default(),
            created: timestamp_to_utc(item.added_at),
            cross_revision_id: item.id,
            id: item.id,
            is_removed: false,
            is_hidden_from_archive: false,
            is_locked: false,
            last_modified: timestamp_to_utc(item.updated_at),
            origin_category: Default::default(),
            revision_id: item.id,
            status: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Enum, Default, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum BookAccessibility {
    #[default]
    Full,
}

#[derive(Debug, Clone, Enum, Default, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum BookStatus {
    #[default]
    Active,
}

#[derive(Debug, Clone, Enum, Default, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum BookOriginCategory {
    #[default]
    Imported,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct ActivePeriod {
    pub from: DateTime<Utc>,
}

impl Default for ActivePeriod {
    fn default() -> Self {
        Self { from: Utc::now() }
    }
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct BookMetadata {
    pub categories: Vec<Uuid>,
    pub cover_image_id: Uuid,
    pub cross_revision_id: Uuid,
    pub current_display_price: ContentDisplayPrice,
    pub current_love_display_price: CurrentLoveDisplayPrice,
    pub description: Option<String>,
    pub download_urls: Vec<String>,
    pub entitlement_id: Uuid,
    pub external_ids: Vec<Uuid>,
    pub genre: Uuid,
    pub is_eligible_for_kobo_love: bool,
    pub is_internet_archive: bool,
    pub is_pre_order: bool,
    pub is_social_enabled: bool,
    pub language: String,
    pub phonetic_pronunciations: PhoneticPronounciations,
    pub publication_date: DateTime<Utc>,
    pub revision_id: Uuid,
    pub title: String,
    pub work_id: Uuid,
    pub contributors: Option<Vec<String>>,
    pub contributor_roles: Option<Vec<KoboSyncedContributorRole>>,
    pub series: Option<KoboSyncedSeries>,
}

impl BookMetadata {
    pub fn try_from_library_item(
        value: LibraryItem,
        download_urls: Vec<String>,
    ) -> Result<Self, anyhow::Error> {
        let authors = value
            .media
            .metadata
            .author_name
            .clone()
            .map(|author| author.split(',').map(|s| s.trim().to_string()).collect());
        Ok(Self {
            categories: vec![Uuid::parse_str("00000000-0000-0000-0000-000000000001")?],
            cover_image_id: value.id,
            cross_revision_id: value.id,
            current_display_price: Default::default(),
            current_love_display_price: Default::default(),
            description: value.media.metadata.description.clone(),
            download_urls,
            entitlement_id: value.id,
            external_ids: vec![],
            genre: Uuid::parse_str("00000000-0000-0000-0000-000000000001")?,
            is_eligible_for_kobo_love: false,
            is_internet_archive: false,
            is_pre_order: false,
            is_social_enabled: true,
            // TODO: guess language more intelligently
            language: value
                .media
                .clone()
                .metadata
                .language
                .unwrap_or("en".to_string()),
            phonetic_pronunciations: PhoneticPronounciations {},
            publication_date: value
                .media
                .metadata
                .get_published_date()
                .unwrap_or_default(),
            revision_id: value.id,
            title: value
                .media
                .metadata
                .title
                .clone()
                .unwrap_or("Untitled".to_string()),
            work_id: value.id,
            contributors: authors.clone(),
            contributor_roles: authors.map(|authors| {
                authors
                    .into_iter()
                    .map(|author| KoboSyncedContributorRole { name: author })
                    .collect()
            }),
            series: None,
        })
    }
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct ContentDisplayPrice {
    pub currency_code: String,
    pub total_amount: f64,
}

impl Default for ContentDisplayPrice {
    fn default() -> Self {
        Self {
            currency_code: "USD".to_string(),
            total_amount: 0.0,
        }
    }
}

#[derive(Debug, Clone, Object, Default, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct CurrentLoveDisplayPrice {
    pub total_amount: f64,
}

#[derive(Debug, Clone, Object, Deserialize)]
pub struct PhoneticPronounciations {}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboPublisher {
    pub imprint: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedContributorRole {
    pub name: String,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedSeries {
    pub name: String,
    pub number: f64,
    pub number_float: f64,
    pub id: Uuid,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedReadingState {
    pub entitlement_id: Uuid,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub priority_timestamp: DateTime<Utc>,
    pub status_info: KoboSyncedStatusInfo,
    pub statistics: KoboSyncedStatistics,
    pub current_bookmark: KoboCurrentBookmark,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedStatusInfo {
    pub last_modified: DateTime<Utc>,
    pub status: KoboSyncedStatus,
    pub times_started_read: f64,
    pub last_time_started_read: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Enum, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum KoboSyncedStatus {
    ReadyToRead,
    Finished,
    Reading,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedStatistics {
    pub last_modified: DateTime<Utc>,
    pub spent_reading_minutes: Option<f64>,
    pub remaining_reading_minutes: Option<f64>,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboCurrentBookmark {
    pub last_modified: DateTime<Utc>,
    pub progress_percent: Option<f64>,
    pub content_source_progress_percent: Option<f64>,
    pub location: Option<KoboCurrentBookmarkLocation>,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboCurrentBookmarkLocation {
    pub value: String,
    #[oai(rename = "Type")]
    pub _type: String,
    pub source: String,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct KoboSyncedBook {
    pub book_entitlement: BookEntitlement,
    pub book_metadata: BookMetadata,
    pub reading_state: Option<KoboSyncedReadingState>,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct NewEntitlement {
    pub new_entitlement: KoboSyncedBook,
}

#[derive(Debug, Clone, Object, Deserialize)]
#[oai(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct ChangedEntitlement {
    pub changed_entitlement: KoboSyncedBook,
}

#[derive(Debug, Clone, Union, Deserialize)]
#[serde(untagged)]
pub enum KoboSyncEntitlement {
    NewEntitlement(NewEntitlement),
    ChangedEntitlement(ChangedEntitlement),
}
