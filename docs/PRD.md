# Product Requirements Document

Status: Draft | Owner: {{owner}} | Version: 0.1 | Last updated: {{YYYY-MM-DD}}
References: FEATURES.md (users U-, resources R-, surfaces S-, journeys J-, features FT-)

FEATURES.md feeds this document. When features are approved or changed there, the requirements below are updated to match, and each requirement lists the features it covers.

## Vision and problem
{{App idea, problem, intended outcome, why it matters}}

## Users
The user list (U-) lives in FEATURES.md. Summarize here who they are and what each needs most.

{{Primary users, needs and key use cases, referring to U- IDs}}

## Platforms
The surfaces each feature is offered on (S-) are listed in FEATURES.md. Record here why the MVP ships on these platforms and which are deliberately deferred.

{{Which platforms the MVP ships on (web, iOS, Android, Windows/macOS/Linux desktop, internal admin/operations tools, API for third parties) and why; platforms deliberately deferred}}

## Objectives and success criteria
- OBJ-001: {{objective and measurable success criterion}}

## MVP scope and functional requirements
Priority uses MoSCoW: Must, Should, Could, Won't. IDs are stable: never renumber or reuse them; mark removed requirements as deprecated.

The Features column lists the FEATURES.md features (FT-) each requirement covers; every approved feature appears in at least one requirement. A requirement that isn't a feature (e.g. "works on every supported platform") writes —. `node scripts/check-features.mjs` checks these links and copies them into FEATURES.md.

| ID | Features | Requirement | Acceptance criterion | Priority |
|---|---|---|---|---|
| FR-001 | FT-001 | {{User/system capability}} | {{Observable result}} | Must |

## Out of scope / stretch
- {{Deferred capability}}

## Business rules and glossary
Resources (R-) in FEATURES.md are the main glossary terms; define them here once, with their rules.

- BR-001: {{Rule, constraints, edge cases}}
- {{Term}}: {{Meaning}}

## Non-functional requirements
- NFR-001: {{Performance target and measurement}}
- {{Accessibility, reliability, privacy, security, data retention targets}}

## Constraints and risks
{{Budget, time, dependencies, domain/legal requirements}}

## Open decisions
- {{Question, owner, decision needed by}}

## Approval
{{Human approver, date, approved revision}}
