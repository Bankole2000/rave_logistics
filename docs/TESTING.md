# Testing and Acceptance

Status: Draft | Owner: {{owner}} | Version: 0.1 | Last updated: {{YYYY-MM-DD}}
References: FEATURES.md, PRD.md, TECHNICAL.md

## Strategy
Test layers per app:

| App | Unit | Integration | End-to-end | Tools |
|---|---|---|---|---|
| {{web}} | {{scope}} | {{scope}} | {{scope}} | {{tools}} |
| {{api}} | {{scope}} | {{scope, e.g. against a real test database}} | — | {{tools}} |
| {{mobile}} | {{scope}} | {{scope}} | {{scope, devices/simulators}} | {{tools}} |
| {{desktop}} | {{scope}} | {{scope}} | {{scope, per operating system}} | {{tools}} |
| infra | {{validate/lint}} | {{plan reviewed in PR}} | — | {{tools}} |

Contract tests: one per provider–consumer pair in the API contracts table (TECHNICAL.md), including server-to-server pairs. {{How each pair is checked against its contract package, e.g. schema validation of responses in provider tests and generated client types in consumers}}. Providers with installed-client consumers (mobile, desktop) also test against the oldest supported contract version.

Test data: isolated per environment; synthetic only.

## Acceptance tests
| ID | Requirement | Features | Apps | Scenario | Expected result |
|---|---|---|---|---|---|
| TC-000 | M0 | — | all | Fresh clone: follow the README setup, run the "All apps" checks and start every app | Every check passes; each client shows live data from the server. A human checks each client in a browser or the app, including the browser console |
| TC-001 | FR-001 | FT-001 | {{web, api}} | {{Action and preconditions}} | {{Observable result}} |

A feature's outcome (FEATURES.md) is the starting point for its expected result. Each journey (J-) gets at least one end-to-end test that walks its features in order. Cover failure cases, validation, authorization and critical user journeys. Use synthetic data; never production secrets or personal data. A journey that crosses apps is tested end to end at least once.

## Validation commands
See the README [Commands](../README.md#commands) section. Record here only how and when each check runs (locally, in CI, before release).

## Definition of done
- [ ] Acceptance criteria met and linked to tests
- [ ] For UI changes: a human checked the change in a browser or the app, including that the browser console shows no errors or warnings (such as hydration mismatches); agents report what to check rather than checking it themselves
- [ ] Automated checks passed for every changed app and every app depending on a changed package
- [ ] No unrelated changes or exposed secrets (including in client app configuration)
- [ ] Documentation and the affected apps' CHANGELOG.md updated where user-visible
- [ ] PR reviewed and merged by authorized human

## Release acceptance
{{Per app: critical journeys, security checks, performance targets, supported browsers/devices/OS versions, known limitations, human sign-off}}

## Approval
{{Human approver, revision, date}}
