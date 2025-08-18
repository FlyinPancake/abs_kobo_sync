// Mapping from ABS DTOs to domain models

use super::models::{Book, FileKind, FileRef, SeriesRef};
use crate::abs_client::{ItemResponse, LibrarySeries};

pub fn map_series(s: &LibrarySeries) -> SeriesRef {
    SeriesRef {
        id: s.id.clone(),
        name: s.name.clone(),
    }
}

pub fn infer_file_kind_from_name(name: &str) -> FileKind {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".epub") {
        FileKind::Epub
    } else if lower.ends_with(".pdf") {
        FileKind::Pdf
    } else if lower.ends_with(".m4b") {
        FileKind::M4b
    } else if lower.ends_with(".mp3") {
        FileKind::Mp3
    } else {
        FileKind::Unknown(lower)
    }
}

pub fn map_item_to_book(abs_base_url: &str, item: &ItemResponse) -> Book {
    // Best effort extraction using flattened extra map until we model more DTO fields
    let title = item.title.clone().unwrap_or_else(|| "Untitled".into());
    let authors = item
        .extra
        .get("authors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let series = item.extra.get("series").and_then(|v| {
        let id = v.get("id")?.as_str()?.to_string();
        let name = v.get("name")?.as_str()?.to_string();
        Some(SeriesRef { id, name })
    });

    let cover_url = Some(format!(
        "{}/api/items/{}/cover",
        abs_base_url.trim_end_matches('/'),
        item.id
    ));

    let formats: Vec<FileRef> = item
        .extra
        .get("tracks")
        .and_then(|v| v.as_array())
        .map(|tracks| {
            tracks
                .iter()
                .filter_map(|t| {
                    let name = t.get("title").and_then(|x| x.as_str()).unwrap_or("");
                    let url = t.get("path").and_then(|x| x.as_str()).unwrap_or("");
                    if url.is_empty() {
                        return None;
                    }
                    Some(FileRef {
                        kind: infer_file_kind_from_name(name),
                        url: url.to_string(),
                        size: None,
                        mime: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Book {
        id: item.id.clone(),
        title,
        authors,
        series,
        cover_url,
        formats,
        description: None,
    }
}
