use poem_openapi::payload::Json;
use uuid::Uuid;

use crate::{
    abs_client::AbsClient,
    kobo_api::models::{
        ErrorDto, LibraryDto, LibraryItemDto, LibraryItemsResponseDto, LibraryListResponse,
    },
};

pub struct LibraryService<'a> {
    pub client: &'a AbsClient,
}

impl<'a> LibraryService<'a> {
    pub fn new(client: &'a AbsClient) -> Self {
        Self { client }
    }

    #[tracing::instrument(level = "debug", skip(self, api_key))]
    pub async fn list_libraries(&self, api_key: &String) -> LibraryListResponse {
        match self.client.get_libraries(api_key).await {
            Ok(libs) => {
                let dtos = libs
                    .libraries
                    .into_iter()
                    .map(|l| LibraryDto {
                        id: l.id,
                        name: l.name,
                        media_type: l.media_type,
                    })
                    .collect();
                LibraryListResponse::Ok(Json(dtos))
            }
            Err(e) => {
                tracing::error!(error = %format!("{:?}", e), "failed to list libraries");
                LibraryListResponse::BadGateway(Json(ErrorDto {
                    message: format!("ABS error: {}", e),
                }))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(level = "debug", skip(self, include, filter))]
    pub async fn list_library_items(
        &self,
        library_id: &Uuid,
        limit: i64,
        page: Option<i64>,
        include: Option<&str>,
        filter: Option<&str>,
        api_key: &String,
    ) -> LibraryItemsResponseDto {
        let res = self
            .client
            .get_library_items(library_id, limit, page, include, filter, api_key)
            .await;

        match res {
            Ok(items) => {
                let dtos: Vec<LibraryItemDto> = items
                    .results
                    .into_iter()
                    .map(|it| {
                        let title = it
                            .media
                            .metadata
                            .title
                            .unwrap_or("Unknown Title".to_string());
                        let author = Some(
                            it.media
                                .metadata
                                .author_name
                                .unwrap_or("Unknown Author".to_string()),
                        );
                        let series = Some(
                            it.media
                                .metadata
                                .series_name
                                .unwrap_or("Unknown Series".to_string()),
                        );
                        let cover_url = Some(it.media.cover_path.unwrap_or("".to_string()));
                        let ebook_format = it.media.ebook_format.as_deref().map(|f| f.to_string());

                        // Prefer using cover_url helper which builds the public URL
                        let computed_cover = Some(self.client.cover_url(&it.id, None, None, false));

                        LibraryItemDto {
                            id: it.id,
                            title,
                            author,
                            series,
                            cover_url: computed_cover.or(cover_url),
                            ebook_format,
                        }
                    })
                    .collect();
                LibraryItemsResponseDto::Ok(Json(dtos))
            }
            Err(e) => {
                tracing::error!(error = %format!("{:?}", e), library_id=%library_id, "failed to list items");
                LibraryItemsResponseDto::BadGateway(Json(ErrorDto {
                    message: format!("ABS error: {}", e),
                }))
            }
        }
    }
}
