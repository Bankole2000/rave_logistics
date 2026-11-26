# Review: project setup

- Reviewer: Bankole
- Date: 2026-11-26
- Completeness: 9 of 9 required facts
- Review status: Partially applied

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

Resolution: Applied 2026-11-26. Set the title and the description (corrected "Let's" to "Lets") and set the license to MIT, with a new LICENSE file. Commands now has one block per app (12) and the example blocks are gone. The commands stay as placeholders until ADR-001 picks the stack.

### AGENTS.md

Project name in Purpose; branch naming and commit convention under "Branches and pull requests".

Resolution: Applied. Set the project name, the branch naming (`<issue number>-<short-title>`) and Conventional Commits scoped by app.

### Document headers in docs/*.md

Set Owner to the document owner and Last updated to the date you apply this review.

Resolution: Applied. Every docs/*.md has Owner @bankole2000 and Last updated 2026-11-26, and each version is bumped (FEATURES.md to 0.3, the others to 0.2).

### docs/GITHUB.md

Repository, Project and Default branch under Configuration; the App labels list (one `app:<name>` per app, plus `app:packages` and `app:repo`).

Resolution: Applied. Set the repository, the default branch, 0 approvals and the 12 `app:` labels, keeping `app:packages`, `app:infra` and `app:repo`. The Project is not set, so it stays a placeholder and the label fallback applies.

### docs/FEATURES.md

Surfaces table: one row per app (S-01, S-02 …: `apps/customer-web`, `apps/customer-app`, `apps/drivers-web`, `apps/drivers-app`, `apps/admin-app`, `apps/admin-web`, `apps/api`, `apps/worker`, `apps/vendor-web`, `apps/vendor-app`, `apps/partners-api`, `apps/mcp-server`): name, `apps/<app>` path, a short description and priority Must; add SDKs or other packages people use directly as further surfaces. Leave the users, actions, resources, journeys and features for the feature review.

Resolution: Applied without reusing IDs. The surfaces map to the 12 apps: S-01 customer-app, S-02 customer-web, S-03 drivers-app, S-04 drivers-web, S-08 vendor-web, S-10 vendor-app (mobile and desktop), S-11 admin-web (now the back office for every staff role, including operations), S-13 admin-app (mobile and desktop), S-14 partners-api, S-15 developer docs (served by partners-api), S-16 mcp-server. Added S-17 api and S-18 worker as Must. S-05, S-06, S-07, S-09 and S-12 are deprecated (Won't), and their features moved to the merged surfaces. FT-016 (payment webhooks) moved from S-14 to S-17. Existing surfaces keep the priorities accepted in the FEATURES.md review rather than all becoming Must.

### docs/TECHNICAL.md

Components table: one row per app and package (and infra if used) with name, path, kind and owner. API contracts table: one row per server app and its consumers. Leave stack and hosting placeholders for the team to decide.

Resolution: Applied. Components has one row per app (12) plus infra, with owner @bankole2000. The example package row is gone because no packages are listed yet. API contracts has rows for api (clients, and server-to-server for partners-api, mcp-server and worker), partners-api and mcp-server. The contract package names, stack, hosting and FR columns stay as placeholders.

### docs/DEPLOYMENT.md

"Where each app runs": one row per app. Approval: release owner.

Resolution: Applied. "Where each app runs" has one row per app, with tag `<app>-v*` triggers and hosts as placeholders. Approval names @bankole2000 as release owner for every app.

### docs/TESTING.md

Strategy table: one row per app (and infra if used).

Resolution: Applied. The Strategy table has one row per app plus infra, with scopes and tools as placeholders.

### .github/CODEOWNERS

Default owner on the `*` line; one line per app and package with its code owner; owners for /infra/, docs/, AGENTS.md and .github/ (default code owner unless told otherwise); the release owner on .github/workflows/release.yml.

Resolution: Applied. @bankole2000 is on `*`, on each app line, and on /packages/, /infra/, /docs/, AGENTS.md and /.github/. The release.yml line moved to the end so the later /.github/ line doesn't override it.

### .github/workflows/ci.yml.example

One path filter and one job per app, named after the app; remove example jobs for apps not listed (keep the infra job only if infra/ is used); add every job to `ci-ok` needs; set the push branch to the default branch.

Resolution: Applied. There is one path filter and one job per app (12), all in `ci-ok` needs, and pushes run on main. The infra job stays until ADR-002 decides whether infra/ is used. The job steps still fail until the commands are configured.

### .github/workflows/release.yml.example

One release job per app (one per platform for mobile, one per operating system for desktop), named after the app, each with environment `release-<app>`; remove example jobs for apps not listed. Creating the environments, their secrets and required reviewers (the release owner) is a human step in GitHub settings.

Resolution: Applied. There is one job per app with environment `release-<app>`. Mobile apps get Android and iOS jobs, and admin-app and vendor-app also get a desktop matrix job. Creating the environments, secrets and reviewers is a human step.

### .github/dependabot.yml.example

Keep only the ecosystems the listed apps use and point directories at their apps.

Resolution: Not applied. The listed apps' ecosystems depend on the stack, which ADR-001 hasn't chosen yet. Apply this after ADR-001.

### GitHub labels, milestones and ruleset (not a file)

A human (or an agent explicitly authorized to change GitHub settings) runs: `bash scripts/setup-github.sh --repo Bankole2000/rave_logistics --apps customer-web,customer-app,drivers-web,drivers-app,admin-app,admin-web,api,worker,vendor-web,vendor-app,partners-api,mcp-server --ruleset --approvals 0` (0 approvals: one person owns everything, and GitHub does not let anyone approve their own pull requests). After CI runs on pull requests, re-run with `--required-check ci-ok`. Milestones come from the docs/ROADMAP.md Milestones table; fill in its slice names first. Record here who ran it.

Resolution: Left for the human, because running setup-github.sh changes GitHub. The ROADMAP.md milestone names are filled in (M0 to M6), so the printed command can run now.
