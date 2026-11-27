# AGENTS.md — api

These instructions add to the repository root `AGENTS.md`; they never relax it. Read the root file first.

## What this app is
- Purpose: the core HTTP api used by every client app, and the only owner of the database. partners-api, mcp-server and worker call it with service credentials.
- Kind: server/API
- Stack: Rust (pinned in the root `rust-toolchain.toml`), axum on tokio, sqlx with PostgreSQL + PostGIS, OpenAPI generated with utoipa. Details in docs/TECHNICAL.md, Components (ADR-003, ADR-006).
- Depends on: PostgreSQL with PostGIS (`docker-compose.yml` at the repo root for local use). Its contract is packages/api-contract.

## Commands
Use this app's block in the README [Commands](../../README.md#commands) section. Run them from the repo root (`cargo … -p api`); `cargo test` needs the local database running and `apps/api/.env`.

## Rules for this app
- Configuration: every variable is read in `src/config.rs` and documented in `.env.example`. Secrets come from the secret store named in docs/DEPLOYMENT.md and are never logged. Never log one-time codes, session tokens or personal data; log phone numbers only as `Phone::masked()`. The only exception is `ConsoleSender`, which prints codes for local development, and api refuses it in release builds.
- Authentication: a handler that needs a signed-in user takes `CurrentSession` (`src/auth/mod.rs`) and declares `security(("session" = []))` in its `#[utoipa::path]`. Read request bodies with `ApiJson` (`src/extract.rs`), not `axum::Json`, so invalid bodies get the standard error body. Store only hashes of codes and tokens (`src/auth/secrets.rs`).
- Staff authorization: a staff-only handler takes `CurrentStaff` and calls `staff.require(Permission::…)` before doing anything, and lists the 403 codes in its `#[utoipa::path]`. Permissions live in `src/auth/roles.rs` and come from the staff rows of the coverage matrix in docs/FEATURES.md; add one there, with its feature, when a feature needs it. Never check a role by name in a handler. Test each staff endpoint with a role that has the permission and one that doesn't.
- Errors: handlers return `ApiResult<T>`; every error response uses the `ErrorBody` shape from `src/error.rs`, with a stable `code`. Add a variant or code there rather than building error JSON in a handler. Internal errors are logged and shown to clients only as `internal_error`.
- Routes: register every handler through `api_router()` in `src/lib.rs` with `routes!(…)` and a `#[utoipa::path]` annotation, so the OpenAPI document can't drift from the code. New endpoints go under `/v1`; `/health` stays unversioned.
- Contract: after changing a route or schema, regenerate the spec with `cargo run -p api --bin openapi -- --out packages/api-contract/openapi.json` and commit it in the same PR. `--check` fails when it is out of date. Keep changes additive within `/v1` (docs/TECHNICAL.md, API compatibility).
- Migrations: SQL files in `migrations/`, named `<UTC timestamp>_<what>.sql`, created with `sqlx migrate add -r <what>` or by hand. Never edit a migration that has been merged; add a new one. api runs pending migrations on start unless `API_MIGRATE_ON_START=false`.
- Tests: HTTP behaviour is tested in `tests/` against the whole app (`api::app`) with `tower::ServiceExt::oneshot`. Use `#[sqlx::test]` for anything touching the database; each test gets its own temporary database with the migrations applied.
- Directories not to edit by hand: none yet. `packages/api-contract/openapi.json` is generated.

## Release
Versioned separately; tags are `api-vX.Y.Z`. Record user-visible changes in this folder's CHANGELOG.md. Release steps are in docs/DEPLOYMENT.md.
