use entities::*;
use poem_openapi::payload::Json;
use sea_orm::{ConnectionTrait, EntityOrSelect, EntityTrait};
use uuid::Uuid;

use crate::{
    AbsKoboResult,
    abs_client::AbsClient,
    kobo_api::models::{ErrorDto, MetadataResponseDto},
};

pub struct MetadataService<'a> {
    pub client: &'a AbsClient,
    pub db: &'a sea_orm::DatabaseConnection,
}

impl<'a> MetadataService<'a> {
    pub fn new(client: &'a AbsClient, db: &'a sea_orm::DatabaseConnection) -> Self {
        Self { client, db }
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

    #[tracing::instrument(level = "debug", skip(self, book_uuid))]
    pub async fn get_metadata(&self, book_uuid: Uuid, auth_token: Uuid) -> MetadataResponseDto {
        let api_key = match self.get_api_key(auth_token).await {
            Ok(Some(api_key)) => api_key,
            _ => {
                return MetadataResponseDto::Unauthorized(Json(ErrorDto {
                    message: "Invalid auth token".into(),
                }));
            }
        };
        let item = match self.client.get_item(book_uuid, false, None, &api_key).await {
            Ok(item) => item,
            Err(_) => {
                return MetadataResponseDto::NotFound(Json(ErrorDto {
                    message: "Item not found".into(),
                }));
            }
        };

        MetadataResponseDto::Ok(Json(todo!()))
    }
}
