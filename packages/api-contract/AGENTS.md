# AGENTS.md — packages/api-contract

These instructions add to the repository root `AGENTS.md`; they never relax it. Read the root file first.

## What this package is
- Purpose: the API contract for apps/api: its OpenAPI 3.1 document, and later the generated TypeScript client (openapi-typescript, openapi-fetch).
- Kind: API contract
- Used by: customer-web, customer-app, drivers-web, drivers-app, vendor-web, vendor-app, admin-web, admin-app, partners-api, mcp-server, worker (as they are created).
- Stack: OpenAPI generated from the Rust handlers with utoipa (ADR-006).

## Rules for this package
- Packages are not deployed or versioned on their own; they ship inside the apps that use them.
- A change here affects every consumer. Run the checks of every app listed under "Used by" (README [Commands](../../README.md#commands) section), not just this package's.
- Breaking changes must be made additive first. The compatibility rules in docs/TECHNICAL.md apply, because released mobile and desktop apps keep using old versions.
- Never import from an app. Never put secrets or environment-specific values here.
- Generated files: `openapi.json` is generated from apps/api with `cargo run -p api --bin openapi -- --out packages/api-contract/openapi.json`. Never edit it by hand; `--check` fails when it is out of date.
