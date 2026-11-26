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
### Changed
### Fixed
### Removed
