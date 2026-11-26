# Technical Specification

Status: Draft | Owner: @bankole2000 | Version: 0.2 | Last updated: 2026-11-26
References: FEATURES.md, PRD.md, DESIGN.md

## Architecture overview
{{How the apps fit together: which clients call which services, data stores, external services. A component diagram helps.}}

## Components
One row per deployable app in `apps/`, per shared package in `packages/`, and for `infra/` if used. App names match folder names, `app:` labels and release tags. Kinds: web client, mobile client, desktop client, server/API, worker, shared package, infrastructure.

| Component | Path | Kind | Stack and version | Deploys to | Owner | Requirements |
|---|---|---|---|---|---|---|
| customer-web | apps/customer-web | Web client | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| customer-app | apps/customer-app | Mobile client | {{stack; minimum OS versions}} | {{App Store / Google Play}} | @bankole2000 | {{FR IDs}} |
| drivers-web | apps/drivers-web | Web client | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| drivers-app | apps/drivers-app | Mobile client | {{stack; minimum OS versions}} | {{App Store / Google Play}} | @bankole2000 | {{FR IDs}} |
| admin-app | apps/admin-app | Mobile client, Desktop client | {{stack; minimum OS versions}} | {{App Store / Google Play; desktop download or store}} | @bankole2000 | {{FR IDs}} |
| admin-web | apps/admin-web | Web client | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| api | apps/api | Server/API | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| worker | apps/worker | Worker | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| vendor-web | apps/vendor-web | Web client | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| vendor-app | apps/vendor-app | Mobile client, Desktop client | {{stack; minimum OS versions}} | {{App Store / Google Play; desktop download or store}} | @bankole2000 | {{FR IDs}} |
| partners-api | apps/partners-api | Server/API | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| mcp-server | apps/mcp-server | Server/API | {{stack}} | {{hosting}} | @bankole2000 | {{FR IDs}} |
| infra | infra/ | Infrastructure | {{IaC tool, see ADR-002}} | {{cloud accounts / environments}} | {{owner}} | — |

## Local development
Defaults that avoid the usual clashes on a developer machine; change them only with a reason.
- **Ports:** uncommon ones, so the apps don't collide with other local tools (3000, 5173, 8000 and 8080 are often taken): {{e.g. api 18080, web 13000}}. Record each app's port in its `.env.example`.
- **Host:** use `127.0.0.1`, not `localhost`, in URLs and bind addresses. `localhost` can resolve to IPv6 first, which adds a delay to every database connection on some systems and makes some dev servers listen on IPv6 only.
- **Databases and other services** run in `docker-compose.yml` (committed) with trust authentication on `127.0.0.1` for local use and CI only, so no password sits in the repository. Each developer's extra services (caches, queues, other databases) go in a gitignored `docker-compose.override.yml`, which Compose loads automatically; commented-out services there cost nothing until needed. Turn off automatic restarts for local containers.
- **CORS:** each server reads the allowed client origins from configuration (e.g. `API_CORS_ORIGINS`), with defaults for every local client: the web dev server and the desktop and mobile shells.

## Repository and workspace
- Monorepo tooling: see ADR-001. {{e.g. pnpm workspaces + Turborepo; uv workspace; Gradle multi-project}}
- Dependency rules: apps may depend on packages; packages never depend on apps; apps never import from another app's folder.
- {{Naming, code style, generated-code locations}}

Commands are listed in the README [Commands](../README.md#commands) section, not here.

## Modules
Significant modules within each component.

| Component | Module | Responsibility | Dependencies | Requirements |
|---|---|---|---|---|
| {{Component}} | {{Module}} | {{Purpose}} | {{Dependencies}} | FR-001 |

## Domain and data model
Start from the resources (R-) in FEATURES.md: each stored resource becomes an entity, "Belongs to" becomes a relationship, and derived resources become queries or views. The users (U-) and the actions each may take (the coverage matrix) become the roles and permissions.

{{Entities, fields, relationships, constraints, indexes, migrations. Which component owns each data store.}}

## API and integrations
| ID | Operation | Input/output | Auth and errors | Requirement |
|---|---|---|---|---|
| API-001 | {{Method/path or operation}} | {{Contract}} | {{Policy/errors}} | FR-001 |

{{External integrations, timeouts, retries, idempotency, events/workflows where needed}}

### API contracts
Every server app has a contract package. List every consumer, including other servers.

| Provider | Contract | Consumers | Consumer auth | Compatibility rule |
|---|---|---|---|---|
| api | packages/{{api-contract}} | customer-web, customer-app, drivers-web, drivers-app, vendor-web, vendor-app, admin-web, admin-app | {{user sessions/tokens}} | {{Installed-client window (below)}} |
| api | packages/{{api-contract}} | partners-api, mcp-server, worker (server-to-server) | {{service credentials, never a user's token}} | {{Rolling-deploy window}} |
| partners-api | packages/{{partners-api-contract}} | External partner systems (U-12) | {{partner API keys}} | {{Public API versioning, e.g. /v1 additive only}} |
| mcp-server | packages/{{mcp-contract}} | AI agents (U-13) | {{agent connection tokens (R-31)}} | {{Tool schema versioning}} |

### API compatibility
- Clients and servers use generated or shared types from the contract package; they don't hand-write request shapes.
- Versioning: {{e.g. additive changes only within /v1; breaking changes need /v2}}.
- Installed clients (mobile, desktop) cannot be force-updated instantly. A provider with installed-client consumers supports every client version released in the last {{N months / N versions}}.
- Server-to-server: services deploy independently, so for the duration of a rollout both the old and new version of each service are live. Every change must work with the previous version of each consumer and provider.
- Breaking a field or endpoint: add the replacement → release consumers that use it → wait out the compatibility window → remove the old one. Record each step in an ADR.
- Forced upgrade: {{how an installed client learns its version is unsupported, e.g. a minimum-version endpoint checked at startup}}.

## Infrastructure
- Tool and layout: see ADR-002 and `infra/AGENTS.md`. {{e.g. modules/ plus environments/<env>/}}
- What it manages: {{networks, databases, hosting, DNS, queues, secret store entries; per environment}}
- What it does not manage: {{e.g. app stores, third-party SaaS configured by hand; list them so nothing is forgotten}}
- State: {{remote backend and locking}}. Applying changes is a release step (DEPLOYMENT.md).

## Third-party services and costs
| Service | Used by | Purpose | Plan / tier | Expected monthly cost | Limits and lock-in notes |
|---|---|---|---|---|---|
| {{Service}} | {{component}} | {{Purpose}} | {{Free / paid tier}} | {{Estimate}} | {{Quotas, exit path}} |

## Error handling
{{How errors are classified, surfaced to users (messages, codes) and to developers; retry and fallback rules; offline behavior for mobile and desktop; what must never be shown to users}}

## Logging and observability
{{What each component logs and at which level, structured format, request/correlation IDs passed from client to server and between services, crash reporting for mobile and desktop, what must never be logged (secrets, personal data), metrics and tracing. Alerting and dashboards are in DEPLOYMENT.md.}}

## Security and privacy
- Client apps (web, mobile, desktop) are public: their code and configuration ship to users. They hold no secrets; anything they can access, an attacker can too. All authorization is enforced server-side.
- Internal apps (e.g. admin, ops) are still public on the internet unless the network says otherwise; they rely on server-side authorization like any other client.
- Server-to-server calls use service credentials scoped to what the caller needs, never a forwarded user token.
- {{Identity, permissions, input validation, secrets storage, sensitive data, threat mitigations, retention}}

## Technical decisions
| ID | Decision | Rationale | Status |
|---|---|---|---|
| ADR-001 | Monorepo tooling: {{workspace tool and task runner}} | {{Alternatives and tradeoffs; mobile toolchain compatibility}} | Proposed |
| ADR-002 | Infrastructure-as-code tool: {{tool, or "none: managed hosting configured by hand"}} | {{Alternatives and tradeoffs; state storage}} | Proposed |
| ADR-003 | {{Decision}} | {{Alternatives and tradeoffs}} | Proposed |

## Open decisions and approval
{{Human approver, revision, unresolved risks}}
