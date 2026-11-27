# GitHub Management Specification

Status: Draft | Owner: @bankole2000 | Version: 0.3 | Last updated: 2026-11-27
<!-- review:focus "Journey approval", "Merging and the risk gate", "Guard rails", "Needs me" -->

## Configuration
- Repository: Bankole2000/rave_logistics
- Project: Bankole2000 #6
- Default branch: main
- Branch protection: a ruleset on the default branch that blocks deletion and force-pushes and requires pull requests with 0 approvals (one person owns everything, and GitHub does not let anyone approve their own pull requests). Once CI runs on pull requests, also require the `ci-ok` status check (it summarizes the per-app CI jobs, which are skipped when their app is unchanged), and once the risk gate has run on a pull request, the `risk-gate` status (see [Merging and the risk gate](#merging-and-the-risk-gate)). Require only these two. Teams also require review from code owners (`.github/CODEOWNERS`). Use least-privilege credentials. Never store tokens in this file.
- `bash scripts/setup-github.sh --all --apps <app1,app2,…> --approvals <n> --project <number>` links the Project and creates its custom fields, the labels, the milestones from ROADMAP.md and the ruleset, and removes GitHub's unused default labels (`--dry-run` first shows what would change). Without a Project, leave out `--project` to get the label fallback, or use `--project-from <number>` to copy a template Project that already has the fields, Status options and views (the owner keeps one, "MVP project template"; its README lists what it holds). The script adds missing Status options (In Review, Blocked) itself while a Project has no items. `--all` also sets the repository to allow auto-merge and delete head branches after merge, and lists GitHub milestones that ROADMAP.md no longer has. Re-run it with `--required-check ci-ok,risk-gate` once both have run on a pull request. App labels come from `--apps`, or from the folders in `apps/` once they exist. In the ruleset UI the check is under Settings → Rules → Rulesets → the ruleset → "Require status checks to pass" → "Add checks" → `ci-ok`.

## Project fields
| Field | Values | New issue default |
|---|---|---|
| Status | Todo, In Progress, In Review, Blocked, Done | Todo |
| Priority | Critical, High, Medium, Low | Medium |
| Complexity | Small, Medium, Large | Set by planner |
| Human Review | Required, Approved, Changes Requested | Required |
| AI Ready | No, Yes | No |
| Journey | J-01 … J-16 (docs/FEATURES.md, Journeys), Platform | Set by planner (`scripts/agent plan`) |

Human Review and AI Ready are separate: only a human approves work; AI Ready indicates sufficient detail. The owner approves a journey at a time (see [Journey approval](#journey-approval)), which sets Human Review on its issues. Material scope changes invalidate approval. Do not use labels that duplicate these fields.

### Without a Project
If the repository has no GitHub Project, the same states are kept in labels, which `setup-github.sh` creates when it runs without `--project`. Every rule in this document and AGENTS.md then reads as follows, and Priority and Complexity go in the issue body.

| Project field | Label instead |
|---|---|
| Status Todo, In Progress, In Review, Blocked | `status:todo`, `status:in-progress`, `status:in-review`, `status:blocked` (exactly one; Done = issue closed) |
| Human Review Approved, Changes Requested | `human:approved`, `human:changes-requested` (only a human adds these; none = Required) |
| AI Ready Yes | `ai:ready` (none = No) |
| Journey | `journey:J-01` … `journey:J-16`, `journey:Platform` (exactly one) |

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
- `app:customer-web`, `app:customer-app`, `app:drivers-web`, `app:drivers-app`, `app:admin-app`, `app:admin-web`, `app:api`, `app:worker`, `app:vendor-web`, `app:vendor-app`, `app:partners-api`, `app:mcp-server` — Work in that app
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

### Merge state
- `risk:high`, `risk:low` — Set by the risk gate on every run, never by hand
- `human:merge-ok` — Only a human adds it: approves merging the head commit. The risk gate removes it when new commits are pushed
- `agent:reviewed` — Added by `scripts/agent review` after it posts a review; adding it re-runs the risk gate. Removed when new commits are pushed

## Issue CRUD and planning
- Read approved specs and existing issues first; use stable task keys to prevent duplicates.
- Create or update only authorized issues; preserve human comments and history. Use native parent/sub-issue and dependency relationships where available, otherwise link issue numbers in the body.
- Create and update issues with `scripts/agent plan --file <plan.json>` (see [Agent workflow](#agent-workflow)). It checks the whole plan before writing anything, then sets milestone and labels, adds each issue to the Project, sets Todo, Human Review Required, Journey, Priority and Complexity, and sets AI Ready from the definition-of-ready check. It also links parents (sub-issues) and dependencies (GitHub issue dependencies, "blocked by"). Re-running a plan updates issues by task key instead of duplicating them. Report any failed step; do not claim success on partial writes.
- Each task belongs to one journey: the journey of its features in docs/FEATURES.md. Work that belongs to no journey (repository tooling, infrastructure, cross-cutting work) is **Platform** work in its milestone; repo and infra tasks without features write "Platform" in place of the FT, FR and TC IDs. A task whose features span journeys is split.
- Human approval remains required: the owner approves journeys, never the planner.
- Close duplicates with a reference only when authorized; prefer closing over deletion. Never delete without explicit authorization.

## Multi-app work
- A feature that needs changes in more than one app gets a parent issue (the user-facing outcome, linked to its FT and FR IDs) with one sub-issue per app. Each sub-issue has a single `app:` label and its own acceptance criteria, and is delivered in its own PR.
- Order sub-issues with dependencies: infrastructure (`app:infra`) first, then the shared contract (`app:packages`), then servers, then clients. Clients may merge behind a feature flag before the server is released.
- The parent issue is Done when all sub-issues are Done and the end-to-end acceptance test passes.

## Journey approval
The owner approves work one journey at a time, not issue by issue: `scripts/agent approve-journey J-12`, or `scripts/agent approve-journey Platform --milestone M1` for a milestone's Platform work. `scripts/agent approve-milestone M1` does this for every journey in M1 and for its Platform work. Approving:
- approves the journey's open tasks that pass the definition-of-ready check (Human Review = Approved, AI Ready = Yes), and lists the ones it skipped and why, so they can be fixed and approved by running it again;
- records the approval on each issue as a comment with a fingerprint of the issue body. Ticking acceptance criteria keeps the approval; any other change to the body undoes it (Human Review back to Required, with a comment) as soon as `next`, `claim`, `ready` or `plan` sees it;
- covers only the tasks open at that moment: tasks added to the journey later stay Required until the owner approves the journey again.

Approving is a human action: the commands refuse to run inside Claude Code and the guard rails block them, so the owner runs them in their own terminal. `--dry-run` shows what would be approved. The milestone's parent issue holds its wave contract (ROADMAP.md) as before.

## Execution lifecycle
Todo + Approved + AI Ready Yes + completed dependencies → In Progress → In Review → Done (after the merge: by a human, or through the risk gate for low-risk PRs). Early start (AGENTS.md, Execution rules 2): a task whose dependency is an open PR may go In Progress for the parts that don't need it; its PR stays a draft until the dependency is Done. Blocked can interrupt any active stage; return to Todo or In Progress as appropriate after resolution. Changes Requested returns the issue to In Progress (see AGENTS.md). Re-review material changes.

## Definition of ready
Clear objective, approved feature IDs (FEATURES.md) and requirement IDs, linked acceptance tests (TC IDs), bounded scope, testable acceptance criteria, known dependencies and resolved decisions.

## PR rules
Use the template, link the issue, report actual validation, identify risks. Use closing keywords only when the PR fully resolves the issue and targets the default branch. Open it with `scripts/agent pr`, which also reports its risk. Who merges depends on the risk (next section); verify issue closure and Project Done afterward.

## Merging and the risk gate
The `risk-gate` workflow (`.github/workflows/risk-gate.yml`) judges every pull request against `.github/risk-rules.json` and reports a `risk-gate` status on its head commit:
- **High risk** when a changed file matches `highRiskPaths` (migrations, `packages/`, `sdks/`, `infra/`, `.github/`, `.claude/`, `scripts/`, `docs/*.md`, AGENTS.md and CLAUDE.md files, dependency manifests and lock files, `.env.example`, docker-compose), a path under `apps/` has a sensitive segment (auth, session, payment, payout, refund, wallet, permission, role, secret), an API contract has a breaking change (removed path, operation, schema or property; newly required field or parameter), or the PR has a `review:*` or `impact:breaking` label. It passes only once a human adds `human:merge-ok`, or the human merges it themselves.
- **Low risk** otherwise. It passes once an agent review of the head commit says `Verdict: Approve` (`scripts/agent review`). With auto-merge on, a low-risk PR then merges by itself when every required check passes.
- New commits remove `human:merge-ok` and `agent:reviewed`, so an approval never carries over to code nobody looked at. Drafts are pending.

The workflow runs on `pull_request_target`: it uses the rules and code of the default branch, so a pull request can't change what judges it, and it never checks out or runs the pull request's code. Changing the rules is itself high risk.

Auto-merge stays off (`"autoMerge": false` in risk-rules.json) until CI runs on pull requests and `ci-ok` is required; otherwise a low-risk PR could merge without its tests. While it is off, the owner merges every PR, and the gate shows which ones an agent has already reviewed.

## Agent workflow
Agents use `scripts/agent` (`node scripts/agent.mjs`; see the README Commands) instead of editing issue fields by hand:
- `next`: the next task, which is Todo, approved and unchanged since, AI Ready, unassigned and not blocked by an open issue; ordered by Priority, milestone, then age.
- `claim <issue>`: checks the same conditions, then creates the branch `<issue>-<title>` on GitHub with a push that fails if the branch already exists, then assigns the issue, comments who claimed it and sets In Progress. Agents share the owner's login, so the assignee can't tell agents apart; the branch is the lock, and of two agents claiming the same task, one gets it. The branch starts with an empty "claim" commit. Set `AGENT_NAME` to name the agent in the claim comment.
- `pr`, `block`, `ask`: open the PR (In Review unless it's a draft); set Blocked with `needs:human-input` and the reason; ask the owner a question on the issue.
- `ready`: re-runs the definition-of-ready check, sets AI Ready, and undoes approvals of changed issues.
- `review <pr> --body-file <file>`: posts a review in the Review output format (AGENTS.md) as a comment review (GitHub doesn't let a PR's author approve it, and agents post as the owner), then adds `agent:reviewed` to re-run the risk gate.
- `risk`: the current branch's risk, as the gate will judge it.
- `digest`: what needs the owner (next section).

## "Needs me" view
Two Project views (Project → **+ New view** → Table; double-click the tab to rename it, type the filter, then **Save**):
- **Needs me**: `is:open label:needs:human-input,risk:high`, which shows questions, blocked tasks and high-risk PRs. A comma means either value. Different qualifiers combine with AND, so one view can't also catch a field value such as Human Review "Changes Requested". Add a third view filtered by `human-review:"Changes Requested"` if you want those listed too.
- **Awaiting approval**: `is:open is:issue human-review:Required ai-ready:Yes`, grouped by Journey (view menu ▾ → Group by → Journey). Field names in filters are lower-case with hyphens; the filter bar autocompletes them.

PRs appear in a view only if they are in the Project. Turn on the built-in **Auto-add to project** workflow with the filter `is:pr is:open`, or high-risk PRs won't show in Needs me.

`scripts/agent digest` prints the same in the terminal: PRs waiting for the owner's merge or `human:merge-ok`, questions and blocked tasks, journeys waiting for approval with the command to approve each, and work in progress.

Turn on the Project's built-in workflows (Project → Workflows): Auto-add to project (`is:pr is:open`), Item added to project → Todo, Item closed → Done, Pull request merged → Done. They keep Status right when work is closed or merged outside `scripts/agent`.

## Guard rails
Agents run in the owner's terminal with the owner's GitHub login and keys, so GitHub can't tell them apart from the owner. Human-only actions are therefore stopped locally and checked again on GitHub:
- `.claude/settings.json` denies them (merging, approving PRs, approving journeys and milestones, pushing to the default branch, force-pushing, tags and releases, triggering workflows, `tofu apply`/`import`/`destroy`, reading or editing `.env` files). Its pre-command hook (`.claude/hooks/guard.mjs`, rules in `scripts/lib/guard.mjs`) blocks the same actions however the command is written, and also adding `human:*` labels and running `setup-github.sh` without `--dry-run`.
- On GitHub: the ruleset (pull requests only, no force-push), the required `ci-ok` and `risk-gate` checks, and the removal of `human:merge-ok` and `agent:reviewed` on new commits.

These stop mistakes. They don't stop someone determined to get round them, who could run the same commands outside Claude Code with the same login. Treat them as safety rails, not access control. A team that needs real separation gives agents their own GitHub identity (a GitHub App or machine user) without merge rights.

## Setup verification
Confirm actual field names/options and IDs, milestone names, labels, permissions, branch rules and automation behavior before agents write. Project fields may require separate API permissions (`gh auth refresh -s project`). This document is a specification, not live GitHub configuration.
