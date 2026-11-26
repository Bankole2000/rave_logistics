# Review: project setup

- Reviewer: Bankole
- Date: 2026-11-26
- Completeness: 9 of 9 required facts
- Review status: Open

> For agents: apply these facts to every file under "Where to apply", replacing the matching placeholders. Expand per-app tables, columns, jobs and lines to exactly the apps listed (remove the template's example apps that are not listed). Replace each "Resolution: pending" with what you changed. Set "Review status" to Applied when all entries are resolved, or Partially applied otherwise. Follow AGENTS.md "Applying doc reviews". Facts marked _Not set_ stay as placeholders.

## Facts

| Fact | Value |
|---|---|
| Project name | Rave Connect Logistics |
| License | MIT |
| One-line description | Let's customers place orders for food and items from nearby restaurants and stores and have them delivered |
| Repository | Bankole2000/rave_logistics |
| Default branch | main |
| GitHub Project number | _Not set_ |
| Document owner | @bankole2000 |
| Release owner | @bankole2000 |
| Default code owner | @bankole2000 |
| Branch naming | 123-short-title |
| Commit convention | Conventional Commits, scoped by app |

## Apps (12)

| App | Path | Kind | Code owner |
|---|---|---|---|
| customer-web | apps/customer-web | Web client | @bankole2000 |
| customer-app | apps/customer-app | Mobile client | @bankole2000 |
| drivers-web | apps/drivers-web | Web client | @bankole2000 |
| drivers-app | apps/drivers-app | Mobile client | @bankole2000 |
| admin-app | apps/admin-app | Mobile client, Desktop client | @bankole2000 |
| admin-web | apps/admin-web | Web client | @bankole2000 |
| api | apps/api | Server/API | @bankole2000 |
| worker | apps/worker | Worker | @bankole2000 |
| vendor-web | apps/vendor-web | Web client | @bankole2000 |
| vendor-app | apps/vendor-app | Mobile client, Desktop client | @bankole2000 |
| partners-api | apps/partners-api | Server/API | @bankole2000 |
| mcp-server | apps/mcp-server | Server/API | @bankole2000 |

## Shared packages (0)

None listed yet.

## Where to apply (13)

### README.md

Title (project name), one-line description and License. Commands: one block per app (plus blocks for packages with their own checks), named after the app; delete the unused example blocks.

Resolution: pending

### AGENTS.md

Project name in Purpose; branch naming and commit convention under "Branches and pull requests".

Resolution: pending

### Document headers in docs/*.md

Set Owner to the document owner and Last updated to the date you apply this review.

Resolution: pending

### docs/GITHUB.md

Repository, Project and Default branch under Configuration; the App labels list (one `app:<name>` per app, plus `app:packages` and `app:repo`).

Resolution: pending

### docs/FEATURES.md

Surfaces table: one row per app (S-01, S-02 …: `apps/customer-web`, `apps/customer-app`, `apps/drivers-web`, `apps/drivers-app`, `apps/admin-app`, `apps/admin-web`, `apps/api`, `apps/worker`, `apps/vendor-web`, `apps/vendor-app`, `apps/partners-api`, `apps/mcp-server`): name, `apps/<app>` path, a short description and priority Must; add SDKs or other packages people use directly as further surfaces. Leave the users, actions, resources, journeys and features for the feature review.

Resolution: pending

### docs/TECHNICAL.md

Components table: one row per app and package (and infra if used) with name, path, kind and owner. API contracts table: one row per server app and its consumers. Leave stack and hosting placeholders for the team to decide.

Resolution: pending

### docs/DEPLOYMENT.md

"Where each app runs": one row per app. Approval: release owner.

Resolution: pending

### docs/TESTING.md

Strategy table: one row per app (and infra if used).

Resolution: pending

### .github/CODEOWNERS

Default owner on the `*` line; one line per app and package with its code owner; owners for /infra/, docs/, AGENTS.md and .github/ (default code owner unless told otherwise); the release owner on .github/workflows/release.yml.

Resolution: pending

### .github/workflows/ci.yml.example

One path filter and one job per app, named after the app; remove example jobs for apps not listed (keep the infra job only if infra/ is used); add every job to `ci-ok` needs; set the push branch to the default branch.

Resolution: pending

### .github/workflows/release.yml.example

One release job per app (one per platform for mobile, one per operating system for desktop), named after the app, each with environment `release-<app>`; remove example jobs for apps not listed. Creating the environments, their secrets and required reviewers (the release owner) is a human step in GitHub settings.

Resolution: pending

### .github/dependabot.yml.example

Keep only the ecosystems the listed apps use and point directories at their apps.

Resolution: pending

### GitHub labels, milestones and ruleset (not a file)

A human (or an agent explicitly authorized to change GitHub settings) runs: `bash scripts/setup-github.sh --repo Bankole2000/rave_logistics --apps customer-web,customer-app,drivers-web,drivers-app,admin-app,admin-web,api,worker,vendor-web,vendor-app,partners-api,mcp-server --ruleset --approvals 0` (0 approvals: one person owns everything, and GitHub does not let anyone approve their own pull requests). After CI runs on pull requests, re-run with `--required-check ci-ok`. Milestones come from the docs/ROADMAP.md Milestones table; fill in its slice names first. Record here who ran it.

Resolution: pending
