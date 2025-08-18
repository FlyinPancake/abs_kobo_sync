use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use poem_openapi::payload::Json;
use serde_json::json;
use uuid::Uuid;

use crate::{
    abs_client::AbsClient,
    kobo_api::models::{
        DeviceAuthResponseDto, EmptyOkResponseDto, InitializationResponseDto, NoContentResponseDto,
        SyncResponseDto, TagCreateRequestDto, TagCreateResponseDto, TagItemDto,
    },
};
// no_std: poem-openapi will serialize headers

pub struct SyncService<'a> {
    pub client: &'a AbsClient,
}

impl<'a> SyncService<'a> {
    pub fn new(client: &'a AbsClient) -> Self {
        Self { client }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn sync(&self) -> SyncResponseDto {
        // Minimal stub: no changes; return empty list with a dummy sync token
        let token = URL_SAFE_NO_PAD.encode("initial");
        SyncResponseDto::Ok(Json(vec![]), token, None)
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
