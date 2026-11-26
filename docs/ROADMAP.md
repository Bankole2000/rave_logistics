# Development Roadmap

Status: Draft | Owner: @bankole2000 | Version: 0.3 | Last updated: 2026-11-26
References: FEATURES.md, PRD.md, DESIGN.md, TECHNICAL.md, TESTING.md

## Milestones
This table is the source of truth for milestone names; GitHub milestones must match exactly. `scripts/setup-github.sh` reads them from here.

M0 (foundation) comes first and the launch milestone last. The milestones in between are slices of the journeys in FEATURES.md: each one delivers whole journeys end to end, so it ends with something a user can try. Cut them from the approved features by priority (Must first) and by "Depends on", then name each one for what a user can do when it is done.

Managing milestones: a milestone is the set of journeys in its Journeys column. To add, split or move one, edit this table (a journey belongs to at most one milestone), then run `node scripts/check-features.mjs --write`. It updates the journeys' Milestone column in FEATURES.md, fails if a feature depends on one in a later milestone, and prints what each milestone delivers. Then `bash scripts/setup-github.sh --milestones` creates new milestones on GitHub and keeps their descriptions (outcome and journeys) in step. A journey too large for one milestone is split into two journeys in FEATURES.md first.

| Milestone | Outcome | Journeys | Exit criteria |
|---|---|---|---|
| M0 - Foundation | Runnable skeleton of every app | — | The M0 checklist below is done; TC-000 passes on a fresh clone |
| M1 - Vendors and drivers can join | Admins set up zones and staff; vendors are approved and publish their catalogues; drivers are approved | J-12, J-05, J-06, J-08 | Features of these journeys pass their acceptance tests on their surfaces |
| M2 - Customers can order and receive deliveries | A customer orders from the nearest store, the vendor prepares it and the nearest driver delivers it | J-01, J-02, J-07, J-09, J-03 | Features of these journeys pass their acceptance tests on their surfaces |
| M3 - Support, operations and payouts | Staff resolve problems and watch live deliveries; vendors and drivers get paid; management sees reports | J-04, J-11, J-10, J-13, J-14 | Features of these journeys pass their acceptance tests on their surfaces |
| M4 - Partners can integrate | E-commerce partners book and track deliveries through the documented Partner API | J-15 | Features of these journeys pass their acceptance tests on their surfaces |
| M5 - Hardening and launch | Reliable, deployed product | — | Acceptance and security checks pass; release checklist approved |
| M6 - Post-launch (stretch) | AI agents order, track and triage through MCP | J-16 | Defined per release |

### M0 checklist
Every project with a client and a server needs these in M0; drop the ones that don't apply.
- [ ] Workspaces and toolchain versions pinned (ADR-001); apps and packages created from `templates/`.
- [ ] Tauri mobile spike (ADR-005). On a real Android and a real iOS device, drivers-app must send its location every `LOCATION_INTERVAL_SECONDS` for 30 minutes while backgrounded and locked, and receive a push notification from api while closed. Record the result in ADR-005; if it fails, drivers-app switches to Capacitor.
- [ ] Each server answers a health check that includes its database, with request IDs and the standard error body.
- [ ] Local defaults (TECHNICAL.md, Local development): uncommon ports, `127.0.0.1`, trust-auth database in `docker-compose.yml`, a gitignored `docker-compose.override.yml` for personal services.
- [ ] Each app's `.env.example` documents every variable; clients get the server URL from public configuration.
- [ ] CORS on each server for every client origin (browser, desktop and mobile shells), from configuration.
- [ ] Generated contracts (API spec, typed clients) are committed, with checks that fail when they are out of date. Generators write their own output files.
- [ ] An api client collection (e.g. Bruno) for manual testing, with a check that it matches the spec.
- [ ] CI runs per app with path filters and one required `ci-ok` check; Dependabot configured.
- [ ] TC-000: on a fresh clone, every README step and command works, and each client shows live data from the server. A human checks each client in a browser or the app, including the browser console.

## Work breakdown
| Task key | Outcome | App | Features | Requirement IDs | Tests | Milestone | Depends on | Priority | Issue |
|---|---|---|---|---|---|---|---|---|---|
| TASK-001 | {{Small testable outcome}} | {{app}} | FT-001 | FR-001 | TC-001 | M1 | {{None or task keys}} | High | TBD |

## Planning rules
- Plan only the next useful milestone in detail; split large tasks. FEATURES.md is the whole map; tasks are created only for the next slice.
- Plan from approved features (FEATURES.md, status `approved`) and their requirements, never from features that are only proposed.
- Each task belongs to one app (or `packages`/`infra`/`repo`). A feature spanning apps becomes one task per app, ordered infrastructure → contract → servers → clients (see GITHUB.md, Multi-app work).
- Tasks that build on a shared interface several tasks use (an api contract, a shared UI component): write the interface down in the milestone's parent issue first ("wave contract"), so the tasks can run in parallel.
- Mobile releases include store review time, and desktop releases need signing certificates and store accounts that can take days to weeks to obtain (see DEPLOYMENT.md). Start those early enough for the launch milestone.
- Set up the release workflow and each app's release environment before its first release, not during it.
- Map every implementation issue to feature IDs, requirement IDs and acceptance tests (TC IDs).
- Record dependencies and issue URLs; GitHub is the source of truth for task status.
- Reconcile scope changes with FEATURES.md and the approved PRD before generating new tasks.

## Approval
{{Human approver, revision, date}}
