# ABS Kobo Sync

Adapter service that bridges a Kobo eReader with your Audiobookshelf (ABS) server. It exposes a small HTTP API that mimics the pieces of Kobo's sync that we can feasibly implement, translating requests into ABS queries and serving metadata, covers, and files back to the device.

Status: early prototype. Current endpoints are just health/test; ABS client exists and can query status, libraries, series.

## Quick start

Prereqs:
- Rust (stable)
- ABS reachable from this service
- Environment variables:
  - `ABS_BASE_URL` (e.g. `http://localhost:13378` or your reverse-proxy base path)
  - `ABS_API_KEY` (create a Read API key in ABS)

Run:

```fish
# In the repo root
cargo build
cargo run
```

By default the API listens on `http://localhost:3000`.

OpenAPI/Docs:
- Spec: `GET /spec`
- UI: `GET /ui`

Basic endpoints now:
- `GET /test` → simple text
- `GET /status` → ABS status passthrough

## Implementation plan (high level)

1) Foundations
- Keep a thin, typed ABS client (reqwest + serde) for status, libraries, series, items, covers, and file downloads.
- Expose a Poem + poem-openapi service with explicit request/response models.
- Add tracing/logging and structured errors.

2) Domain and mapping
- Define domain models that are device-agnostic (Book, Author, Series, Cover, FileRef, Progress).
- Implement mappers from ABS JSON to domain models.
- Decide on stable IDs for device consumption (prefer ABS item id; maintain a mapping if device needs another format).

3) Minimal Kobo-facing surface (to validate device workflows)
- Library listing: list purchased/owned books → return mapped domain collection.
- Metadata: book detail, series, author, cover URLs.
- File serving: stream file(s) for a book item or its EPUB/PDF if available.
- Progress endpoints: capture read position updates and serve last-known position on sync.

4) State and auth
- Persist device registrations, tokens (if any), and per-book progress locally (SQLite via `sqlx` recommended for simplicity).
- Provide a simple device token or shared secret auth for write endpoints (progress updates), configurable via env.

5) Performance & robustness
- Cache ABS responses for library/metadata (in-memory LRU + ETag/If-None-Match when available).
- Range requests and partial content for file streaming.
- Timeouts, retries, and circuit breakers toward ABS.

6) Developer experience & tests
- Unit tests for mapping logic.
- Contract tests for ABS client with example payloads.
- Golden samples for API responses.

7) Deployment
- Container image (small, static where possible).
- Config via env; health endpoints; readiness checks.

See `docs/IMPLEMENTATION_PLAN.md` for a more detailed breakdown with milestones.

## Suggested code structure

- `src/main.rs`
  - Startup: config, tracing, build routes, serve OpenAPI
- `src/config.rs`
  - Env-driven config (ABS base url/key, listen addr, feature flags)
- `src/abs_client/` (already present)
  - Typed client and DTOs for ABS endpoints
- `src/kobo_api/` (already present)
  - HTTP handlers for device-facing endpoints (Poem OpenAPI)
  - Split into `routes.rs` and `models.rs` as it grows
- `src/domain/`
  - `models.rs` → device-agnostic book/series/author/file/progress types
  - `mapping.rs` → ABS → domain mappers
- `src/storage/`
  - `mod.rs` + `sqlite.rs` → repository traits + SQLite implementation (device, progress, item map)
- `src/services/`
  - `sync.rs` → library sync/cache refresh jobs
  - `progress.rs` → apply and fetch reading progress
- `src/errors.rs`
  - AppError and conversions to HTTP status
- `src/telemetry.rs`
  - tracing, request IDs, logging middleware

Optional later:
- `src/cache/` for a dedicated in-memory cache abstraction
- `src/http_utils.rs` for range requests, stream helpers

## Configuration

Environment variables (current + planned):
- Current
  - `ABS_BASE_URL` (required)
  - `ABS_API_KEY` (required)
- Planned
  - `BIND_ADDR` (default `0.0.0.0:3000`)
  - `DEVICE_SHARED_SECRET` (optional for auth to progress endpoints)
  - `CACHE_TTL_SECONDS` (default 300)
  - `DATABASE_URL` (e.g., `sqlite://abs_kobo_sync.db`)

## Roadmap

- [ ] ABS client: items, search, file download, covers (public URLs already available)
- [ ] Domain models and mappers
- [ ] Storage: sqlite repositories for device + progress
- [ ] Kobo API: list library, item detail, cover, file download
- [ ] Progress: get/set endpoints
- [ ] Caching and range support
- [ ] Observability (tracing + structured logs)
- [ ] Containerization and deployment docs

## Troubleshooting

- Ensure `ABS_BASE_URL` is reachable from this process.
- Verify `ABS_API_KEY` has permission to read libraries/items.
- Use `/spec` and `/ui` to validate the API is up.
