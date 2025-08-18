use poem_openapi::payload::Json;
use serde_json::json;

use crate::{
	abs_client::AbsClient,
	kobo_api::models::{ErrorDto, ReadingStateGetResponseDto, ReadingStatePutResponseDto},
};

pub struct ReadingService<'a> {
	pub client: &'a AbsClient,
}

impl<'a> ReadingService<'a> {
	pub fn new(client: &'a AbsClient) -> Self {
		Self { client }
	}

	#[tracing::instrument(level = "debug", skip(self, book_uuid))]
	pub async fn get_state(&self, book_uuid: &str) -> ReadingStateGetResponseDto {
		if uuid::Uuid::parse_str(book_uuid).is_err() {
			return ReadingStateGetResponseDto::NotFound(Json(ErrorDto { message: "Invalid book UUID".into() }));
		}
		let state = json!({
			"EntitlementId": book_uuid,
		});
		ReadingStateGetResponseDto::Ok(Json(vec![state]))
	}

	#[tracing::instrument(level = "debug", skip(self, book_uuid, payload))]
	pub async fn update_state(&self, book_uuid: &str, payload: serde_json::Value) -> ReadingStatePutResponseDto {
		if uuid::Uuid::parse_str(book_uuid).is_err() {
			return ReadingStatePutResponseDto::BadRequest(Json(ErrorDto { message: "Invalid book UUID".into() }));
		}
		// Basic validation for required fields
		let first = payload
			.get("ReadingStates")
			.and_then(|v| v.as_array())
			.and_then(|arr| arr.get(0));
		let cb = first.and_then(|st| st.get("CurrentBookmark"));
		let has_location = cb.and_then(|c| c.get("Location")).is_some();
		let has_cspp = cb
			.and_then(|c| c.get("ContentSourceProgressPercent"))
			.and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
			.is_some();
		if !has_location || !has_cspp {
			return ReadingStatePutResponseDto::BadRequest(Json(ErrorDto { message: "Missing Location or ContentSourceProgressPercent".into() }));
		}
		let result = json!({
			"RequestResult": "Success",
			"UpdateResults": [
				{
					"EntitlementId": book_uuid,
					"CurrentBookmarkResult": { "Result": "Success" },
					"StatisticsResult": { "Result": "Ignored" },
					"StatusInfoResult": { "Result": "Success" }
				}
			]
		});
		let _ = payload; // unused for now
		ReadingStatePutResponseDto::Ok(Json(result))
	}
}

