use poem_openapi::payload::Json;

use crate::{
	abs_client::AbsClient,
	kobo_api::models::{ErrorDto, MetadataResponseDto},
};

pub struct MetadataService<'a> {
	pub client: &'a AbsClient,
}

impl<'a> MetadataService<'a> {
	pub fn new(client: &'a AbsClient) -> Self {
		Self { client }
	}

	#[tracing::instrument(level = "debug", skip(self, book_uuid))]
	pub async fn get_metadata(&self, book_uuid: &str) -> MetadataResponseDto {
		// Minimal stub: return a single object with key fields per spec
		let rid = match uuid::Uuid::parse_str(book_uuid) {
			Ok(u) => u,
			Err(_) => return MetadataResponseDto::NotFound(Json(ErrorDto{ message: "Invalid book UUID".into()})),
		};
		let obj = serde_json::json!({
			"Categories": [],
			"ContributorRoles": [],
			"Contributors": [],
			"CoverImageId": rid,
			"CrossRevisionId": rid,
			"CurrentDisplayPrice": { "CurrencyCode": "USD", "TotalAmount": 0 },
			"CurrentLoveDisplayPrice": { "TotalAmount": 0 },
			"Description": serde_json::Value::Null,
			"DownloadUrls": [],
			"EntitlementId": rid,
			"ExternalIds": [],
			"Genre": "00000000-0000-0000-0000-000000000001",
			"IsEligibleForKoboLove": false,
			"IsInternetArchive": false,
			"IsPreOrder": false,
			"IsSocialEnabled": true,
			"Isbn": serde_json::Value::Null,
			"Language": "en",
			"PhoneticPronunciations": {},
			"PublicationDate": serde_json::Value::Null,
			"Publisher": serde_json::Value::Null,
			"RevisionId": rid,
			"Series": serde_json::Value::Null,
			"Slug": serde_json::Value::Null,
			"SubTitle": serde_json::Value::Null,
			"Title": rid.to_string(),
			"WorkId": rid
		});
		MetadataResponseDto::Ok(Json(vec![obj]))
	}
}

