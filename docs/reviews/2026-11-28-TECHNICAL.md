# Review: docs/TECHNICAL.md

- Document: `docs/TECHNICAL.md` (status Draft, version 0.5 at time of review)
- Reviewer: Bankole
- Date: 2026-11-28
- Verdict: Approved
- Coverage: 81 of 81 items decided; 2 need action; 0 added; 0 with placeholder values; 2 boilerplate
- Review status: Applied

> For agents: apply every entry under "Action required", "Added items" and "Placeholder values" to `docs/TECHNICAL.md`, then replace each "Resolution: pending" with what you did (or why not). Set "Review status" to Applied when all entries are resolved, or Partially applied otherwise. Leave accepted items unchanged. Line numbers refer to the document as reviewed; find items by ID or text if it has changed. Follow AGENTS.md "Applying doc reviews".

## Action required (2)

### ADR-001 · Modify

Section: Technical decisions · Line 150

Original:

```
| ADR-001 | Monorepo tooling: pnpm workspaces and a Cargo workspace at the root, with Turborepo as the task runner across both. | The repo has two languages, and each is best served by its own native workspace. Tauri's `src-tauri` crates join the Cargo workspace like any other crate. Turborepo adds caching and `--filter` per app for CI and local work, and thin `package.json` wrappers give Rust apps the same commands. Alternatives: Nx (more features, but heavier and its Rust support is a plugin); `just` alone (simple, but no caching or affected-app filtering); separate repositories (contracts drift between them). | Proposed |
```

**Suggested change:** Use `just` alone. No seperate repositories. We should be able to use it on the monorepo.

Resolution: Applied in revision 0.6. ADR-001 (Accepted) keeps one repository with pnpm and Cargo workspaces and makes `just` the only task runner, from a root `justfile`; Turborepo moved to the alternatives and separate repositories are rejected. "Repository and workspace" now describes `just` recipes, and CI picks affected apps with path filters.

### ADR-007 · Modify

Section: Technical decisions · Line 156

Original:

```
| ADR-007 | Background jobs and live updates: a job queue in PostgreSQL (`FOR UPDATE SKIP LOCKED`, e.g. apalis with its Postgres backend) consumed by worker. Live updates over WebSockets from api, fanned out between api instances with PostgreSQL `LISTEN/NOTIFY`. | No extra infrastructure for the MVP, and a job is created in the same transaction as the change that needs it. Add Redis (or a managed queue) only when measured load needs it. Alternative: Redis or a managed queue from the start (more moving parts and cost). | Proposed |
```

**Suggested change:** lets use redis and a managed queue from the start - for human observability and insights.

Resolution: Applied in revision 0.6 as the owner clarified it on 2026-11-28: Redis alone (local in docker-compose, the host's managed Redis when deployed) for the job queue with a jobs dashboard, live-update fan-out and driver positions, fed by a PostgreSQL outbox so jobs stay transactional. RabbitMQ and cloud queues are recorded as alternatives. The architecture overview, diagram, worker row and local ports match. TECHNICAL stays In review because this wording and the owner's other decisions of 2026-11-28 (operating system versions, provider stand-ins) were not in the reviewed revision.

## Added items (0)

None.

## Placeholder values (0)

None.

## Unfilled placeholders (32)

- "customer-web" (line 37): `{{hosting}}`, `{{FR IDs}}`
- "customer-app" (line 38): `{{version}}`, `{{version}}`, `{{App Store / Google Play}}`, `{{FR IDs}}`
- "drivers-web" (line 39): `{{hosting}}`, `{{FR IDs}}`
- "drivers-app" (line 40): `{{version}}`, `{{version}}`, `{{App Store / Google Play}}`, `{{FR IDs}}`
- "admin-app" (line 41): `{{version}}`, `{{version}}`, `{{version}}`, `{{version}}`, `{{App Store / Google Play; desktop download or store}}`, `{{FR IDs}}`
- "admin-web" (line 42): `{{hosting}}`, `{{FR IDs}}`
- "api" (line 43): `{{hosting}}`, `{{FR IDs}}`
- "worker" (line 44): `{{hosting}}`, `{{FR IDs}}`
- "vendor-web" (line 45): `{{hosting}}`, `{{FR IDs}}`
- "vendor-app" (line 46): `{{version}}`, `{{version}}`, `{{version}}`, `{{version}}`, `{{App Store / Google Play; desktop download or store}}`, `{{FR IDs}}`
- "partners-api" (line 47): `{{hosting}}`, `{{FR IDs}}`
- "mcp-server" (line 48): `{{hosting}}`, `{{FR IDs}}`
- "infra" (line 54): `{{cloud accounts / environments}}`
- "{{Component}}" (line 85): `{{Component}}`, `{{Module}}`, `{{Purpose}}`, `{{Dependencies}}`
- "{{Other entities, fields, relationships, constraints and indexes, as they are added.}}" (line 94): `{{Other entities, fields, relationships, constraints and indexes, as they are added.}}`
- API-001 (line 99): `{{Method/path or operation}}`, `{{Contract}}`, `{{Policy/errors}}`
- "{{External integrations, timeouts, retries, idempotency, events/workflows where needed}}" (line 101): `{{External integrations, timeouts, retries, idempotency, events/workflows where needed}}`
- "api" (line 108): `{{user sessions/tokens}}`, `{{Installed-client window (below)}}`
- "api" (line 109): `{{service credentials, never a user's token}}`, `{{Rolling-deploy window}}`
- "partners-api" (line 110): `{{partner API keys}}`, `{{Public API versioning, e.g. /v1 additive only}}`
- "mcp-server" (line 111): `{{agent connection tokens (R-31)}}`, `{{Tool schema versioning}}`
- "Installed clients (mobile, desktop) cannot be force-updated instantly. A provider with in…" (line 116): `{{N months / N versions}}`
- "Forced upgrade: {{how an installed client learns its version is unsupported, e.g. a minim…" (line 119): `{{how an installed client learns its version is unsupported, e.g. a minimum-version endpoint checked at startup}}`
- "Tool and layout: see ADR-002 and `infra/AGENTS.md`. {{e.g. modules/ plus environments/<en…" (line 122): `{{e.g. modules/ plus environments/<env>/}}`
- "What it manages: {{networks, databases, hosting, DNS, queues, secret store entries; per e…" (line 123): `{{networks, databases, hosting, DNS, queues, secret store entries; per environment}}`
- "What it does not manage: {{e.g. app stores, third-party SaaS configured by hand; list the…" (line 124): `{{e.g. app stores, third-party SaaS configured by hand; list them so nothing is forgotten}}`
- "State: {{remote backend and locking}}. Applying changes is a release step (DEPLOYMENT.md)." (line 125): `{{remote backend and locking}}`
- "{{Service}}" (line 130): `{{Service}}`, `{{component}}`, `{{Purpose}}`, `{{Free / paid tier}}`, `{{Estimate}}`, `{{Quotas, exit path}}`
- "{{How errors are classified, surfaced to users (messages, codes) and to developers; retry…" (line 133): `{{How errors are classified, surfaced to users (messages, codes) and to developers; retry and fallback rules; offline behavior for mobile and desktop; what must never be shown to users}}`
- "{{What each component logs and at which level, structured format, request/correlation IDs…" (line 136): `{{What each component logs and at which level, structured format, request/correlation IDs passed from client to server and between services, crash reporting for mobile and desktop, what must never be logged (secrets, personal data), metrics and tracing. Alerting and dashboards are in DEPLOYMENT.md.}}`
- "{{Input validation, secrets storage, sensitive data, threat mitigations, retention}}" (line 145): `{{Input validation, secrets storage, sensitive data, threat mitigations, retention}}`
- "{{Human approver, revision}}" (line 167): `{{Human approver, revision}}`

## Accepted (79)

69 of these were accepted in bulk ("Accept all"), without being opened one by one.

- "Proposed (ADR-001 to ADR-007). All eight client apps use one Vue UI stack. The four web a…" (line 8)
- "flowchart LR" (line 10)
- "Live tracking (FR-010, FR-024) goes over WebSockets from `api`. Driver positions (R-10) a…" (line 30)
- "One row per deployable app in `apps/`, per shared package in `packages/`, and for `infra/…" (line 33)
- "customer-web" (line 37)
- "customer-app" (line 38)
- "drivers-web" (line 39)
- "drivers-app" (line 40)
- "admin-app" (line 41)
- "admin-web" (line 42)
- "api" (line 43)
- "worker" (line 44)
- "vendor-web" (line 45)
- "vendor-app" (line 46)
- "partners-api" (line 47)
- "mcp-server" (line 48)
- "api-contract" (line 49)
- "partners-api-contract" (line 50)
- "mcp-contract" (line 51)
- "ui" (line 52)
- "domain" (line 53)
- "infra" (line 54)
- "Defaults that avoid the usual clashes on a developer machine; change them only with a rea…" (line 57)
- "**Ports:** uncommon ones, so the apps don't collide with other local tools (3000, 5173, 8…" (line 58)
- "**Host:** use `127.0.0.1`, not `localhost`, in URLs and bind addresses. `localhost` can r…" (line 59)
- "**Databases and other services** run in `docker-compose.yml` (committed) with trust authe…" (line 60)
- "**CORS:** each server reads the allowed client origins from configuration (e.g. `API_CORS…" (line 61)
- "Monorepo tooling: see ADR-001. There are two workspaces at the root and one task runner a…" (line 64)
- "Dependency rules: apps may depend on packages; packages never depend on apps; apps never …" (line 68)
- "Code style:" (line 69)
- "Generated code is committed, and each generator writes its own output file:" (line 72)
- "Toolchains are pinned: `rust-toolchain.toml`, `.nvmrc` and `packageManager` in the root `…" (line 76)
- "Commands are listed in the README [Commands](../README.md#commands) section, not here." (line 78)
- "Significant modules within each component." (line 81)
- "{{Component}}" (line 85)
- "Start from the resources (R-) in FEATURES.md: each stored resource becomes an entity, "Be…" (line 88)
- "api owns the database; other components go through its API. Migrations are in `apps/api/m…" (line 90)
- "**users, sign_in_codes and sessions** (R-30, ADR-008): `users` is one row per person (a v…" (line 92)
- "**staff_members and staff_invitations** (R-22): a staff member is a user with one role (s…" (line 93)
- "{{Other entities, fields, relationships, constraints and indexes, as they are added.}}" (line 94)
- API-001 — {{Method/path or operation}} (line 99)
- "{{External integrations, timeouts, retries, idempotency, events/workflows where needed}}" (line 101)
- "Every server app has a contract package. List every consumer, including other servers." (line 104)
- "api" (line 108)
- "api" (line 109)
- "partners-api" (line 110)
- "mcp-server" (line 111)
- "Clients and servers use generated or shared types from the contract package; they don't h…" (line 114)
- "Versioning: additive changes only within `/v1`; breaking changes need `/v2`. This applies…" (line 115)
- "Installed clients (mobile, desktop) cannot be force-updated instantly. A provider with in…" (line 116)
- "Server-to-server: services deploy independently, so for the duration of a rollout both th…" (line 117)
- "Breaking a field or endpoint: add the replacement → release consumers that use it → wait …" (line 118)
- "Forced upgrade: {{how an installed client learns its version is unsupported, e.g. a minim…" (line 119)
- "Tool and layout: see ADR-002 and `infra/AGENTS.md`. {{e.g. modules/ plus environments/<en…" (line 122)
- "What it manages: {{networks, databases, hosting, DNS, queues, secret store entries; per e…" (line 123)
- "What it does not manage: {{e.g. app stores, third-party SaaS configured by hand; list the…" (line 124)
- "State: {{remote backend and locking}}. Applying changes is a release step (DEPLOYMENT.md)." (line 125)
- "{{Service}}" (line 130)
- "{{How errors are classified, surfaced to users (messages, codes) and to developers; retry…" (line 133)
- "{{What each component logs and at which level, structured format, request/correlation IDs…" (line 136)
- "Client apps (web, mobile, desktop) are public: their code and configuration ship to users…" (line 139)
- "Internal apps (e.g. admin, ops) are still public on the internet unless the network says …" (line 140)
- "Server-to-server calls use service credentials scoped to what the caller needs, never a f…" (line 141)
- "**Identity and sessions** (ADR-008): everyone signs in with a phone number and a one-time…" (line 142)
- "**Email sign-in:** codes go to an email address only when it belongs to a user or has a p…" (line 143)
- "**Staff roles and permissions:** each staff role has a fixed list of permissions (`apps/a…" (line 144)
- "{{Input validation, secrets storage, sensitive data, threat mitigations, retention}}" (line 145)
- ADR-002 — Infrastructure-as-code: OpenTofu, with remote state and locking in the chosen cloud's sto… (line 151)
- ADR-003 — Backend: Rust (stable) with axum on tokio, sqlx and PostgreSQL with PostGIS for api, work… (line 152)
- ADR-004 — Web clients: Vue 3 + TypeScript single-page apps built with Vite. No server-side renderin… (line 153)
- ADR-005 — Installed apps: Tauri 2 with a Vue UI for customer-app, drivers-app, admin-app and vendor… (line 154)
- ADR-006 — API contracts: OpenAPI 3.1 generated from the Rust handlers with utoipa and committed in … (line 155)
- ADR-008 — Authentication: one identity per person (`users`, keyed by verified phone number), signed… (line 157)
- "Open decisions:" (line 160)
- "Hosting provider and region, chosen for latency to Nigeria, cost, and managed PostgreSQL …" (line 161)
- "Minimum Android, iOS, Windows and macOS versions (Components table). Owner: @bankole2000." (line 162)
- "Payment, SMS/one-time-code, maps and push providers (PRD, Open decisions)." (line 163)
- "Unresolved risk: Tauri mobile background location and push for drivers-app (ADR-005), set…" (line 165)
- "{{Human approver, revision}}" (line 167)

## Not shown (2)

- 2 boilerplate items (document header, introduction, "How this file works", "Approval"), approved with the document.

## Not reviewed (0)

None.

<!-- review-baseline {"text|Architecture overview|403f3e72":"403f3e72","text|Architecture overview|d23e4f74":"d23e4f74","text|Architecture overview|f3d45df1":"f3d45df1","text|Components|fceda1c7":"fceda1c7","row|Components|3a5544f5":"3a5544f5","row|Components|b0721038":"b0721038","row|Components|2589e21c":"2589e21c","row|Components|0e102f68":"0e102f68","row|Components|a59c0e34":"a59c0e34","row|Components|5b47b3c8":"5b47b3c8","row|Components|da9fe057":"da9fe057","row|Components|3212a3d9":"3212a3d9","row|Components|b46c7846":"b46c7846","row|Components|03fc6b30":"03fc6b30","row|Components|5d89a91c":"5d89a91c","row|Components|21afe036":"21afe036","row|Components|0df8d15f":"0df8d15f","row|Components|038dc265":"038dc265","row|Components|637d5675":"637d5675","row|Components|08036a9e":"08036a9e","row|Components|4390c354":"4390c354","row|Components|a1cf5044":"a1cf5044","text|Local development|159a3856":"159a3856","list|Local development|e5068d7f":"e5068d7f","list|Local development|351cb8c1":"351cb8c1","list|Local development|ef24e9cd":"ef24e9cd","list|Local development|42b98ccd":"42b98ccd","list|Repository and workspace|c060f19c":"c060f19c","list|Repository and workspace|15a279ee":"15a279ee","list|Repository and workspace|10900bf7":"10900bf7","list|Repository and workspace|4c658f93":"4c658f93","list|Repository and workspace|b3e10666":"b3e10666","text|Repository and workspace|ae57bfc6":"ae57bfc6","text|Modules|366c2890":"366c2890","row|Modules|9432d0bf":"9432d0bf","text|Domain and data model|15d83ca2":"15d83ca2","text|Domain and data model|e317e5ce":"e317e5ce","list|Domain and data model|54005226":"54005226","list|Domain and data model|974e8b5e":"974e8b5e","list|Domain and data model|2c7fa10c":"2c7fa10c","API-001":"66c95058","text|API and integrations|f7327d0a":"f7327d0a","text|API and integrations › API contracts|e596ca9b":"e596ca9b","row|API and integrations › API contracts|07512a71":"07512a71","row|API and integrations › API contracts|0886e553":"0886e553","row|API and integrations › API contracts|624878ce":"624878ce","row|API and integrations › API contracts|2474cf0b":"2474cf0b","list|API and integrations › API compatibility|a7e07f4a":"a7e07f4a","list|API and integrations › API compatibility|37e9298e":"37e9298e","list|API and integrations › API compatibility|a20203b9":"a20203b9","list|API and integrations › API compatibility|5ac9e0e0":"5ac9e0e0","list|API and integrations › API compatibility|961d1ade":"961d1ade","list|API and integrations › API compatibility|83b78623":"83b78623","list|Infrastructure|277079da":"277079da","list|Infrastructure|900d8167":"900d8167","list|Infrastructure|f0a65619":"f0a65619","list|Infrastructure|0ee7ad77":"0ee7ad77","row|Third-party services and costs|50d4e878":"50d4e878","text|Error handling|acde69e5":"acde69e5","text|Logging and observability|8a94fc9b":"8a94fc9b","list|Security and privacy|f16d23e3":"f16d23e3","list|Security and privacy|e2d6ed00":"e2d6ed00","list|Security and privacy|c014d2fb":"c014d2fb","list|Security and privacy|b67e2948":"b67e2948","list|Security and privacy|44f49e17":"44f49e17","list|Security and privacy|23eff72b":"23eff72b","list|Security and privacy|e660b385":"e660b385","ADR-002":"7f12d362","ADR-003":"39620cb8","ADR-004":"59cc4e1b","ADR-005":"f157ab5a","ADR-006":"ddd6953a","ADR-008":"5dbb9488","text|Open decisions and approval|c45f96e4":"c45f96e4","list|Open decisions and approval|da0e11fb":"da0e11fb","list|Open decisions and approval|56db3f64":"56db3f64","list|Open decisions and approval|15ed0e02":"15ed0e02","text|Open decisions and approval|6142cb78":"6142cb78","text|Open decisions and approval|144604f6":"144604f6"} -->
