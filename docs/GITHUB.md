# GitHub Management Specification

Status: Draft | Owner: {{owner}} | Version: 0.1 | Last updated: {{YYYY-MM-DD}}

## Configuration
- Repository: {{owner/repo}}
- Project: {{owner, project number or ID}}
- Default branch: {{main}}
- Branch protection: a ruleset on the default branch that blocks deletion and force-pushes and requires pull requests with {{0 approvals for a solo owner, or 1+ for a team}}. Once CI runs on pull requests, also require the `ci-ok` status check, and only that one (it summarizes the per-app CI jobs, which are skipped when their app is unchanged). Teams also require review from code owners (`.github/CODEOWNERS`). Use least-privilege credentials. Never store tokens in this file.
- `bash scripts/setup-github.sh --all --apps <app1,app2,…> --approvals <n> --project <number>` links the Project and creates its custom fields, the labels, the milestones from ROADMAP.md and the ruleset, and removes GitHub's unused default labels (`--dry-run` first shows what would change). Without a Project, leave out `--project` to get the label fallback, or use `--project-from <number>` to copy a template Project that already has the fields, Status options and views (the owner keeps one, "MVP project template"; its README lists what it holds). The script adds missing Status options (In Review, Blocked) itself while a Project has no items. Re-run it with `--required-check ci-ok` once CI runs. App labels come from `--apps`, or from the folders in `apps/` once they exist. In the ruleset UI the check is under Settings → Rules → Rulesets → the ruleset → "Require status checks to pass" → "Add checks" → `ci-ok`.

## Project fields
| Field | Values | New issue default |
|---|---|---|
| Status | Todo, In Progress, In Review, Blocked, Done | Todo |
| Priority | Critical, High, Medium, Low | Medium |
| Complexity | Small, Medium, Large | Set by planner |
| Human Review | Required, Approved, Changes Requested | Required |
| AI Ready | No, Yes | No |

Human Review and AI Ready are separate: only a human approves work; AI Ready indicates sufficient detail. Material scope changes invalidate approval. Do not use labels that duplicate these fields.

### Without a Project
If the repository has no GitHub Project, the same states are kept in labels, which `setup-github.sh` creates when it runs without `--project`. Every rule in this document and AGENTS.md then reads as follows, and Priority and Complexity go in the issue body.

| Project field | Label instead |
|---|---|
| Status Todo, In Progress, In Review, Blocked | `status:todo`, `status:in-progress`, `status:in-review`, `status:blocked` (exactly one; Done = issue closed) |
| Human Review Approved, Changes Requested | `human:approved`, `human:changes-requested` (only a human adds these; none = Required) |
| AI Ready Yes | `ai:ready` (none = No) |

### Priority
PRD requirements use MoSCoW (Must, Should, Could, Won't) to define scope. Issue Priority orders the work. Default mapping, which the planner may adjust with a stated reason:

| PRD | Issue Priority |
|---|---|
| Must | High (Critical if it blocks other work or users) |
| Should | Medium |
| Could | Low |
| Won't | Not planned; stays out of scope |

## Milestones
Milestones are defined in [ROADMAP.md](ROADMAP.md#milestones). GitHub milestone titles must match them exactly.

## Labels
Use exactly one `type:` label, the `app:` label of each app the issue changes (normally one; see [Multi-app work](#multi-app-work)), `area:` labels where relevant, and optional flags. `app:` says which deployable changes; `area:` says which concern within it. Add project-specific areas only when needed. Labels never duplicate Project fields.

### Type
- `type:feature` — New capability
- `type:bug` — Incorrect behavior
- `type:enhancement` — Improve existing behavior
- `type:refactor` — Internal restructuring
- `type:documentation` — Documentation
- `type:test` — Tests
- `type:chore` — Maintenance
- `type:research` — Investigation
- `type:design` — Design work
- `type:security` — Security work
- `type:performance` — Performance work

### App
One per app in `apps/`, plus one for shared code and one for repository-wide work. Names match the folder names.
- `app:{{app 1, e.g. web}}`, `app:{{app 2, e.g. api}}`, `app:{{app 3, e.g. mobile}}` — Work in that app
- `app:packages` — Shared code in `packages/`
- `app:infra` — Infrastructure-as-code in `infra/`
- `app:repo` — Repository-wide work (CI, tooling, docs)

### Area
- `area:frontend`, `area:backend`, `area:mobile`, `area:desktop`, `area:database`, `area:api`, `area:auth`, `area:integrations`, `area:infrastructure`, `area:devops`

### Flags
- `ai:planned` — Created by planning agent
- `needs:human-input` — Agent is waiting on a human answer
- `needs:specification` — Missing requirements
- `needs:decision` — Unresolved decision
- `review:security`, `review:privacy`, `review:legal`, `review:medical`, `review:financial` — Specialist review required before approval or merge
- `impact:breaking` — Compatibility impact

## Issue CRUD and planning
- Read approved specs and existing issues first; use stable task keys to prevent duplicates.
- Create or update only authorized issues; preserve human comments and history. Use native parent/sub-issue and dependency relationships where available, otherwise link issue numbers in the body.
- Set milestone and labels, add to Project, set Todo, Human Review Required and AI Ready No. Verify each operation and report failures; do not claim success on partial writes.
- Populate Priority and Complexity; mark AI Ready Yes only after checking the definition of ready. Human approval remains required.
- Close duplicates with a reference only when authorized; prefer closing over deletion. Never delete without explicit authorization.

## Multi-app work
- A feature that needs changes in more than one app gets a parent issue (the user-facing outcome, linked to its FT and FR IDs) with one sub-issue per app. Each sub-issue has a single `app:` label and its own acceptance criteria, and is delivered in its own PR.
- Order sub-issues with dependencies: infrastructure (`app:infra`) first, then the shared contract (`app:packages`), then servers, then clients. Clients may merge behind a feature flag before the server is released.
- The parent issue is Done when all sub-issues are Done and the end-to-end acceptance test passes.

## Execution lifecycle
Todo + Approved + AI Ready Yes + completed dependencies → In Progress → In Review → Done (after approved merge). Early start (AGENTS.md, Execution rules 2): a task whose dependency is an open PR may go In Progress for the parts that don't need it; its PR stays a draft until the dependency is Done. Blocked can interrupt any active stage; return to Todo or In Progress as appropriate after resolution. Changes Requested returns the issue to In Progress (see AGENTS.md). Re-review material changes.

## Definition of ready
Clear objective, approved feature IDs (FEATURES.md) and requirement IDs, linked acceptance tests (TC IDs), bounded scope, testable acceptance criteria, known dependencies and resolved decisions.

## PR rules
Use the template, link the issue, report actual validation, identify risks. Use closing keywords only when the PR fully resolves the issue and targets the default branch. A human approves the merge; verify issue closure and Project Done afterward.

## Setup verification
Confirm actual field names/options and IDs, milestone names, labels, permissions, branch rules and automation behavior before agents write. Project fields may require separate API permissions (`gh auth refresh -s project`). This document is a specification, not live GitHub configuration.
