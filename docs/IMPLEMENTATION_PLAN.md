# ABS Kobo Sync – Detailed Plan

This document expands the README plan into concrete milestones, interfaces, and data models.

## Goals

- Bridge Kobo device sync flows to an Audiobookshelf library
- Read-only library and metadata first; add progress write-back later
- Be resilient, cache-friendly, and simple to deploy

## Milestones

1) Core plumbing
- Config, error handling, tracing
- ABS client: status, libraries, series, item, cover URLs
- OpenAPI service wired, health endpoints

2) Domain + mapping
- Introduce domain models
- Implement ABS → Domain mappers and unit tests

3) Library and item APIs
- List library (paged)
- Get item detail (book-level), including series, authors, durations/pages if present
- Cover URL passthrough
- File URL or streaming endpoint

4) Progress
- Storage layer with SQLite
- Endpoints: get/set progress
- Device token/shared secret auth middleware

5) Performance polish
- In-memory cache for library and metadata (LRU + TTL)
- Range requests in streaming
- Timeout/retry strategy toward ABS

6) Packaging & Ops
- Container image, health checks, readiness
- Minimal Grafana/Prometheus hints or logs

## Interfaces

### Config
- ABS_BASE_URL: string
- ABS_API_KEY: string
- BIND_ADDR: string (default 0.0.0.0:3000)
- DEVICE_SHARED_SECRET: string optional
- CACHE_TTL_SECONDS: u64 (default 300)
- DATABASE_URL: string (sqlite://...)

### Errors
- AppError with variants: BadRequest, NotFound, Upstream, Unauthorized, Internal
- Implement IntoResponse for Poem

### Domain Models (src/domain/models.rs)
- Book { id, title, authors: Vec<String>, series: Option<SeriesRef>, cover_url: Option<String>, formats: Vec<FileRef>, description: Option<String> }
- SeriesRef { id, name }
- FileRef { kind: FileKind, url: String, size: Option<u64>, mime: Option<String> }
- FileKind: Epub | Pdf | M4b | Mp3 | Unknown(String)
- Progress { book_id, position: f64, updated_at }

### ABS DTOs
- Keep in `src/abs_client` and only expose what we need

### Mapping (src/domain/mapping.rs)
- from_abs_item(item: abs::ItemResponse) -> Book
- from_abs_series(res: abs::LibrarySeries) -> SeriesRef

### Storage (src/storage/*)
- trait ProgressRepo { get(book_id, device_id) -> Option<Progress>; set(progress) }
- trait DeviceRepo { get_or_register(device_fingerprint) -> Device }
- sqlite implementation using `sqlx` with feature `runtime-tokio-rustls`

### API (src/kobo_api/*)
- models.rs: request/response OpenAPI types
- routes.rs: handlers
- Endpoints (initial pass):
  - GET /v1/library?page=&limit=
  - GET /v1/items/{id}
  - GET /v1/items/{id}/cover -> redirect or proxy to ABS cover
  - GET /v1/items/{id}/file -> stream with Range support
  - GET /v1/progress/{id}
  - PUT /v1/progress/{id}

## Notes on Kobo specifics

- Kobo expects specific endpoints and payload shapes for its own cloud; we’re not fully replicating Kobo cloud. Instead, provide a minimal companion app/endpoint that the device or a companion script can use to pull/push data.
- If later you want tighter Kobo device integration, we can add a separate compatibility layer or a Calibre integration.

## Testing strategy

- Mapping tests with sample ABS payloads (add JSON fixtures under tests/data)
- API tests against the Poem app with `poem::test::TestClient`
- Storage tests with a temp SQLite database

## Risks & mitigations

- ABS API changes: pin to known ABS versions; keep DTOs lenient using `#[serde(flatten)]`
- Large files: enable streaming and range; avoid buffering whole files in memory
- Auth: start with shared secret; revisit JWT/OAuth if needed

## Next steps (immediate)

- [ ] Add domain and mapping skeleton modules
- [ ] Expand ABS client with items search/list and file download URL helper
- [ ] Sketch API models and 2-3 endpoints stubbed with TODOs
