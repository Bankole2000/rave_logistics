# Features

Status: Draft | Owner: {{owner}} | Version: 0.1 | Last updated: {{YYYY-MM-DD}}
References: PRD.md (requirements FR-), ROADMAP.md (milestones)

## How this file works
This file is the list of everything the product lets someone do. It is drafted first and feeds the other documents: when it changes, the PRD requirements, DESIGN, TECHNICAL, TESTING and ROADMAP are updated to match, and they refer back to these IDs.

1. **Lists first.** Users, actions, resources and surfaces are short lists with stable IDs. Every feature is built from them, so the same thing always has the same name. Their details feed the other documents: users become roles and permissions, resources the data model and glossary, actions the api operations and audit events, surfaces the apps.
2. **A feature is one sentence:** *user* + *action* + *resource*, with a *scope* where it matters ("own posts only"), on one or more *surfaces*. Its *outcome* is what the user sees when it worked; it becomes the acceptance criterion.
3. **Journeys** group features in the order a user meets them. Milestones are cut from journeys, so each milestone delivers whole journeys end to end (ROADMAP.md).
4. **Status** records the decision only: `proposed`, `approved`, `deferred` or `out-of-scope`. Whether a feature is built is tracked in GitHub through its tasks, not here.
5. **Effective priority** is the lower of the feature's priority and its surface's priority (a Must feature on a Should surface is a Should there).
6. **IDs are stable:** never renumber or reuse them. Mark removed items `out-of-scope` instead of deleting them.
7. **Only humans approve.** Agents add new ideas as `proposed` and never change a status.
8. **Adding features:** open this file in `tools/doc-reviewer.html` and use "+ Add". "Build from lists" picks the user, action, resource, surfaces and journey from dropdowns ("+ New" adds a list item on the spot); "Describe in words" takes a sentence that an agent turns into a row.
9. **Generated columns:** the Requirements column (from the PRD), the journeys' Milestone column (from ROADMAP.md) and the coverage matrix are written by `node scripts/check-features.mjs --write`. Don't edit them by hand; CI fails when they are out of date.

## Users
Kinds: `person`, `app` (another program using the product), `agent` (a script or AI agent acting for someone) or `system` (the product itself: checks, calculations, scheduled jobs).

| ID | Name | Kind | Description | Priority |
|---|---|---|---|---|
| U-01 | {{Main user, e.g. Customer}} | person | {{Who they are and what they come for}} | Must |
| U-02 | {{Second user, e.g. Admin}} | person | {{Who they are and what they come for}} | Must |
| U-03 | System | system | The product itself: checks on every request, calculations and scheduled jobs. | Must |

## Actions
Common verbs are listed; add the ones this product needs (e.g. `requests`, `accepts`, `pays`, `invites`) and remove those it doesn't.

| ID | Verb | Meaning | Writes data |
|---|---|---|---|
| A-01 | creates | Adds a new record. | yes |
| A-02 | views | Lists records or opens one. | no |
| A-03 | edits | Changes fields of an existing record, including its status. | yes |
| A-04 | archives | Hides a record without deleting it; can be undone. | yes |
| A-05 | removes | Deletes a record permanently. | yes |
| A-06 | searches | Finds records by text. | no |
| A-07 | filters | Narrows a list to records matching a field. | no |
| A-08 | signs in | Starts a session. | yes |
| A-09 | signs out | Ends a session. | yes |
| A-10 | rejects | Refuses a request that breaks a rule, with a stable error code. | no |
| A-11 | records | Stores a record as a side effect of another action (e.g. an audit entry). | yes |

## Resources
Kinds: `stored` (kept in the database) or `derived` (calculated from stored data, e.g. a report or a total).

| ID | Name | Kind | Belongs to | Description |
|---|---|---|---|---|
| R-01 | {{Main resource, e.g. Post}} | stored | — | {{What it is and its key fields}} |
| R-02 | {{Second resource, e.g. Comment}} | stored | {{R-01, or —}} | {{What it is and its key fields}} |

## Surfaces
Where a feature is offered: one row per app in `apps/` (from the project setup), plus SDKs or other packages people use directly.

| ID | Name | App or package | Description | Priority |
|---|---|---|---|---|
| S-01 | {{Surface, e.g. Customer web app}} | apps/{{web}} | {{Who uses it and where}} | Must |
| S-02 | {{Surface, e.g. Public api}} | apps/{{api}} | {{Who uses it and where}} | Must |

## Journeys
A journey is what one user does to reach a goal, as features in order. The Milestone column is generated from ROADMAP.md, where each milestone lists the journeys it delivers.

| ID | Name | Main user | Goal | Features, in order | Milestone |
|---|---|---|---|---|---|
| J-01 | {{Journey, e.g. Publish a post}} | U-01 | {{What the user wants to achieve}} | FT-001 | {{M1}} |

## Features
| ID | User | Action | Resource | Scope | Surfaces | Outcome | Priority | Depends on | Journey | Requirements | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|
| FT-001 | U-01 {{Main user}} | A-01 creates | R-01 {{Main resource}} | {{Scope, or —}} | S-01 | {{What the user sees when it worked}} | Must | — | J-01 | FR-001 | proposed |

## Coverage matrix
Generated from the Features table (approved and proposed features only); don't edit it by hand. Each cell lists the actions a user can take on a resource. An empty cell is not an error, but it is worth a look: who can change or remove this resource?

<!-- matrix:start -->
| User \ Resource | {{Main resource, e.g. Post}} |
|---|---|
| U-01 {{Main user, e.g. Customer}} | creates |
| U-02 {{Second user, e.g. Admin}} |  |
| U-03 System |  |
<!-- matrix:end -->

## Gaps
Questions found while building the lists that the features don't answer yet. Each needs a decision: add a feature (as `proposed`), or note here that it is intentionally missing.

- None yet.

## Not features
Requirements that are not user + action + resource stay in the PRD: quality targets (NFR-), constraints, and "works on every surface" requirements, which the Surfaces list covers.

## Approval
{{Human approver, revision, date}}
