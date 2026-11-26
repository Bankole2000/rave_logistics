# Development Roadmap

Status: Draft | Owner: {{owner}} | Version: 0.1 | Last updated: {{YYYY-MM-DD}}
References: FEATURES.md, PRD.md, DESIGN.md, TECHNICAL.md, TESTING.md

## Milestones
This table is the source of truth for milestone names; GitHub milestones must match exactly. `scripts/setup-github.sh` reads them from here.

M0 (foundation) comes first and the launch milestone last. The milestones in between are slices of the journeys in FEATURES.md: each one delivers whole journeys end to end, so it ends with something a user can try. Cut them from the approved features by priority (Must first) and by "Depends on", then name each one for what a user can do when it is done.

| Milestone | Outcome | Journeys | Exit criteria |
|---|---|---|---|
| M0 - Foundation | Runnable skeleton of every app | — | The M0 checklist below is done; TC-000 passes on a fresh clone |
| M1 - {{First slice, e.g. Customers can publish}} | {{What a user can do when it is done}} | {{J-01, J-02}} | {{Features of these journeys pass their acceptance tests on their surfaces}} |
| M2 - {{Next slice}} | {{What a user can do when it is done}} | {{J-03}} | {{Features of these journeys pass their acceptance tests on their surfaces}} |
| M3 - Hardening and launch | Reliable, deployed product | — | Acceptance and security checks pass; release checklist approved |
| M4 - Post-launch (stretch) | Improvements | {{Deferred journeys}} | Defined per release |

### M0 checklist
Every project with a client and a server needs these in M0; drop the ones that don't apply.
- [ ] Workspaces and toolchain versions pinned (ADR-001); apps and packages created from `templates/`.
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
