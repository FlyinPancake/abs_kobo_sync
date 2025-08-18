use poem_openapi::payload::Json;

use crate::{
    abs_client::AbsClient,
    kobo_api::models::{ErrorDto, LibraryDto, LibraryItemDto, LibraryItemsResponseDto, LibraryListResponse},
};

pub struct LibraryService<'a> {
    pub client: &'a AbsClient,
}

impl<'a> LibraryService<'a> {
    pub fn new(client: &'a AbsClient) -> Self {
        Self { client }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn list_libraries(&self) -> LibraryListResponse {
        match self.client.get_libraries().await {
            Ok(libs) => {
                let dtos = libs
                    .libraries
                    .into_iter()
                    .map(|l| LibraryDto { id: l.id, name: l.name, media_type: l.media_type })
                    .collect();
                LibraryListResponse::Ok(Json(dtos))
            }
            Err(e) => {
                tracing::error!(error = %format!("{:?}", e), "failed to list libraries");
                LibraryListResponse::BadGateway(Json(ErrorDto { message: format!("ABS error: {}", e) }))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(level = "debug", skip(self, include, filter))]
    pub async fn list_library_items(
        &self,
        library_id: &str,
        limit: i64,
        page: Option<i64>,
        include: Option<&str>,
        filter: Option<&str>,
    ) -> LibraryItemsResponseDto {
        let res = self
            .client
            .get_library_items(library_id, limit, page, include, filter)
            .await;

        match res {
            Ok(items) => {
                let dtos: Vec<LibraryItemDto> = items
                    .results
                    .into_iter()
                    .map(|it| {
                        let (title, author, series, cover_url, ebook_format) = if let Some(media) = it.media {
                            let title = media.metadata.as_ref().and_then(|m| m.title.clone());
                            let author = media.metadata.as_ref().and_then(|m| m.author_name.clone());
                            let series = media.metadata.as_ref().and_then(|m| m.series_name.clone());
                            let cover_url = media.cover_path.map(|p| p);
                            let ebook_format = media.ebook_format;
                            (title, author, series, cover_url, ebook_format)
                        } else {
                            (None, None, None, None, None)
                        };

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
                LibraryItemsResponseDto::BadGateway(Json(ErrorDto { message: format!("ABS error: {}", e) }))
            }
        }
    }
}
