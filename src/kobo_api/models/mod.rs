pub mod kobo;
pub use kobo::*;

use std::ffi::os_str::Display;

use poem_openapi::{ApiResponse, Enum, Object, payload::Json};
use uuid::Uuid;

#[derive(Debug, Clone, Object)]
pub struct LibraryDto {
    pub id: Uuid,
    pub name: String,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Object)]
pub struct LibraryItemDto {
    pub id: Uuid,
    pub title: String,
    pub author: Option<String>,
    pub series: Option<String>,
    pub cover_url: Option<String>,
    pub ebook_format: Option<String>,
}

#[derive(Debug, Clone, Object)]
pub struct ErrorDto {
    /// Human-readable error message
    pub message: String,
}

impl From<String> for ErrorDto {
    fn from(message: String) -> Self {
        ErrorDto { message }
    }
}

#[derive(ApiResponse)]
pub enum LibraryListResponse {
    /// Libraries successfully retrieved
    #[oai(status = 200)]
    Ok(Json<Vec<LibraryDto>>),

    /// Upstream ABS error
    #[oai(status = 502)]
    BadGateway(Json<ErrorDto>),
}

#[derive(ApiResponse)]
pub enum LibraryItemsResponseDto {
    /// Items successfully retrieved
    #[oai(status = 200)]
    Ok(Json<Vec<LibraryItemDto>>),

    /// Upstream ABS error
    #[oai(status = 502)]
    BadGateway(Json<ErrorDto>),
}

// ===== Kobo sync and device-facing DTOs (minimal, JSON passthrough where shapes vary) =====

#[derive(ApiResponse)]
pub enum SyncResponseDto {
    /// Sync items successfully retrieved
    #[oai(status = 200)]
    Ok(
        Json<Vec<kobo::KoboSyncEntitlement>>,
        #[oai(header = "X-Kobo-SyncToken")] String,
        #[oai(header = "X-Kobo-Sync")] Option<String>,
        #[oai(header = "X-Kobo-Sync-Mode")] Option<String>,
        #[oai(header = "X-Kobo-Recent-Reads")] Option<String>,
    ),

    /// Unauthorized
    #[oai(status = 401)]
    Unauthorized(Json<ErrorDto>),

    /// Forbidden
    #[oai(status = 403)]
    Forbidden(Json<ErrorDto>),

    /// Upstream or mapping error
    #[oai(status = 502)]
    BadGateway(Json<ErrorDto>),
}

#[derive(ApiResponse)]
pub enum MetadataResponseDto {
    /// One metadata object wrapped in an array
    #[oai(status = 200)]
    Ok(Json<BookMetadata>),

    #[oai(status = 401)]
    Unauthorized(Json<ErrorDto>),

    /// Not found or upstream error
    #[oai(status = 404)]
    NotFound(Json<ErrorDto>),
}

#[derive(ApiResponse)]
pub enum ReadingStateGetResponseDto {
    /// One reading state object wrapped in an array
    #[oai(status = 200)]
    Ok(Json<Vec<serde_json::Value>>),

    #[oai(status = 404)]
    NotFound(Json<ErrorDto>),
}

#[derive(ApiResponse)]
pub enum ReadingStatePutResponseDto {
    /// Update result object
    #[oai(status = 200)]
    Ok(Json<serde_json::Value>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorDto>),
}

#[derive(Debug, Clone, Object)]
pub struct TagItemDto {
    #[oai(rename = "Type")]
    pub r#type: Option<String>,
    #[oai(rename = "RevisionId")]
    pub revision_id: Option<Uuid>,
}

#[derive(Debug, Clone, Object)]
pub struct TagCreateRequestDto {
    #[oai(rename = "Name")]
    pub name: String,
    #[oai(rename = "Items")]
    pub items: Option<Vec<TagItemDto>>,
}

#[derive(Debug, Clone, Object)]
pub struct TagItemsRequestDto {
    #[oai(rename = "Items")]
    pub items: Vec<TagItemDto>,
}

#[derive(ApiResponse)]
pub enum TagCreateResponseDto {
    /// Created with tag id returned as JSON string
    #[oai(status = 201)]
    Created(Json<String>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorDto>),
}

#[derive(ApiResponse)]
pub enum EmptyOkResponseDto {
    /// Empty 200 response
    #[oai(status = 200)]
    Ok,
}

#[derive(ApiResponse)]
pub enum NoContentResponseDto {
    /// Empty 204 response
    #[oai(status = 204)]
    NoContent,
}

#[derive(ApiResponse)]
pub enum InitializationResponseDto {
    /// Initialization resources
    #[oai(status = 200)]
    Ok(Json<serde_json::Value>),
}

#[derive(ApiResponse)]
pub enum DeviceAuthResponseDto {
    /// Synthetic device auth result
    #[oai(status = 200)]
    Ok(Json<serde_json::Value>),
}

#[derive(ApiResponse)]
pub enum NotImplementedResponseDto {
    /// Feature not implemented yet
    #[oai(status = 501)]
    NotImplemented(Json<ErrorDto>),
}

#[derive(Debug, Clone, Enum)]
pub enum BookFormatDto {
    Epub,
    Kepub,
}

impl ToString for BookFormatDto {
    fn to_string(&self) -> String {
        match self {
            BookFormatDto::Epub => "epub".into(),
            BookFormatDto::Kepub => "kepub".into(),
        }
    }
}
