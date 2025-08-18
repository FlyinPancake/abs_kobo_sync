# ABS Kobo Sync – instructions for AI coding agents

## Big picture
- Goal: expose a minimal HTTP API mimicking Kobo sync and translate it to Audiobookshelf (ABS) calls.
- Tech: Rust + Tokio, Poem + poem-openapi for HTTP/OpenAPI, reqwest + serde for ABS client.
- Data flow: Kobo request -> `kobo_api` route -> ABS client (`abs_client`) -> optional `domain` mapping -> HTTP response. Keep ABS DTOs separate from device-facing models.

## Code structure (where things live)
- `src/main.rs` bootstraps config, builds ABS client, mounts OpenAPI routes (`/`, `/ui`, `/spec`).
- `src/config.rs` loads env (`ABS_BASE_URL`, `ABS_API_KEY`) and validates; `.env.local` preferred over `.env` in `main.rs`.
- `src/abs_client/mod.rs` typed ABS client + DTOs. Uses serde renames and `#[serde(flatten)]` to be resilient to upstream changes.
- `src/kobo_api/`
  - `routes.rs` Poem OpenAPI handlers (thin). Depend only on domain types or small DTOs defined in `kobo_api/models.rs`.
  - `models.rs` response DTOs for device-facing API.
- `src/domain/` device-agnostic models and mappers (`mapping.rs`) from ABS DTOs.
- `src/storage/` repo traits; SQLite impl planned (via `sqlx`).

## Conventions and patterns
- ABS DTOs live in `abs_client`; device/API DTOs live in `kobo_api/models.rs`; map to domain in `domain/`.
- Prefer `Option<T>` + `#[serde(rename = "fieldName")]` + `#[serde(flatten)]` for ABS shapes; don’t make fields mandatory unless guaranteed by ABS.
- Public HTTP API via poem-openapi: return typed ApiResponse enums (see `LibraryListResponse`). Keep handlers small and push logic to mappers/services.
- Build cover/file URLs via helpers on `AbsClient` instead of reconstructing strings in routes.
- Add minimal, targeted unit tests next to the code (see tests in `abs_client/mod.rs`); use real sample payloads where possible.

## Developer workflows
- Build/run (Fish shell):
  ```fish
  cargo build
  cargo run
  ```
- OpenAPI: `GET /spec` (JSON) and `GET /ui` (RapiDoc).
- Env: set `ABS_BASE_URL` and `ABS_API_KEY`; `main.rs` auto-loads from `.env.local` or `.env`.

## Adding ABS client features (example)
- Add a method on `AbsClient` (e.g., `get_library_items(...) -> LibraryItemsResponse`). Use `self.url(...)`, include optional query params, attach Authorization if present, and deserialize with serde.
- Create DTOs in `abs_client/mod.rs` with serde renames matching ABS JSON (example: `mediaType` -> `media_type`). Keep lenient with `Option` and `#[serde(flatten)]`.

## Adding API endpoints (example)
- Define request/response DTOs in `kobo_api/models.rs`.
- Implement an `#[oai]` handler in `kobo_api/routes.rs`, call the `AbsClient`, map to domain or passthrough, and return an ApiResponse variant.
- For covers/files, prefer redirecting or constructing ABS URLs via `AbsClient::cover_url`.

## Testing and safety
- Use unit tests with real sample JSON (see `library_items_deserialize_example`) to lock ABS shapes.
- Avoid panics in handlers; return structured errors with an `ErrorDto` and map upstream issues to `502` when appropriate.
- Keep changes small and typed; prefer adding types over ad-hoc JSON access.
