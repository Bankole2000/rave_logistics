# Changelog — api

All notable changes to this app are recorded here. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/) and are tagged `api-vX.Y.Z`.

## [Unreleased]
### Added
- `GET /health`, reporting api and database status (200 when healthy, 503 when the database can't be reached).
- Standard error body (`{"error": {"code", "message"}}`) for every error, including unknown routes.
- `x-request-id` on every response; a request ID sent by the client is kept.
- CORS for the origins in `API_CORS_ORIGINS`.
- Database migrations, run on start; the first one installs PostGIS.
- OpenAPI document generated to `packages/api-contract/openapi.json`.
- Sign-in with a Nigerian phone number and a one-time code (`POST /v1/auth/codes`, `POST /v1/auth/sessions`); the first sign-in creates the account. Codes are limited per number (resend wait and hourly limit) and per code (attempts, expiry).
- Sessions as revocable bearer tokens; `GET /v1/me` returns the signed-in user and `DELETE /v1/auth/sessions/current` signs the device out. Sessions unused for `AUTH_SESSION_IDLE_DAYS` expire, and a suspended user's sessions stop working.
- Error codes `unauthorized` (401), `too_many_requests` (429, with `Retry-After`), `invalid_body`, `invalid_phone`, `invalid_code`, `code_expired` and `account_suspended`.
### Changed
### Fixed
### Removed
