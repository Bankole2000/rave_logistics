# Testing and Acceptance

Status: Draft | Owner: @bankole2000 | Version: 0.3 | Last updated: 2026-11-27
References: FEATURES.md, PRD.md, TECHNICAL.md

## Strategy
Test layers per app:

| App | Unit | Integration | End-to-end | Tools |
|---|---|---|---|---|
| customer-web | {{scope}} | {{scope}} | {{scope}} | {{tools}} |
| customer-app | {{scope}} | {{scope}} | {{scope, devices/simulators}} | {{tools}} |
| drivers-web | {{scope}} | {{scope}} | {{scope}} | {{tools}} |
| drivers-app | {{scope}} | {{scope}} | {{scope, devices/simulators}} | {{tools}} |
| admin-app | {{scope}} | {{scope}} | {{scope, devices/simulators and each desktop operating system}} | {{tools}} |
| admin-web | {{scope}} | {{scope}} | {{scope}} | {{tools}} |
| api | {{scope}} | {{scope, e.g. against a real test database}} | — | {{tools}} |
| worker | {{scope}} | {{scope, e.g. against a real test database}} | — | {{tools}} |
| vendor-web | {{scope}} | {{scope}} | {{scope}} | {{tools}} |
| vendor-app | {{scope}} | {{scope}} | {{scope, devices/simulators and each desktop operating system}} | {{tools}} |
| partners-api | {{scope}} | {{scope, e.g. against a real test database}} | — | {{tools}} |
| mcp-server | {{scope}} | {{scope, e.g. against a real test database}} | — | {{tools}} |
| infra | {{validate/lint}} | {{plan reviewed in PR}} | — | {{tools}} |

Contract tests: one per provider–consumer pair in the API contracts table (TECHNICAL.md), including server-to-server pairs. {{How each pair is checked against its contract package, e.g. schema validation of responses in provider tests and generated client types in consumers}}. Providers with installed-client consumers (mobile, desktop) also test against the oldest supported contract version.

Test data: isolated per environment; synthetic only.

## Acceptance tests
| ID | Requirement | Features | Apps | Scenario | Expected result |
|---|---|---|---|---|---|
| TC-000 | M0 | — | all | Fresh clone: follow the README setup, run the "All apps" checks and start every app | Every check passes; each client shows live data from the server. A human checks each client in a browser or the app, including the browser console |
| TC-001 | FR-001, FR-002, FR-003, FR-036 | FT-001, FT-002, FT-003, FT-004, FT-005, FT-097, FT-101 | customer-app, customer-web, api | J-01: a new phone number signs up with the one-time code, saves and edits an address, signs out, signs in again, signs in with a linked social account, then deletes the account | Account works with the code and with the linked social account; the address is offered at checkout; after deletion sign-in fails and personal data is gone |
| TC-002 | FR-004, FR-005, FR-006, FR-007, FR-008, FR-009 | FT-006, FT-007, FT-008, FT-009, FT-010, FT-011, FT-012, FT-013, FT-014, FT-015, FT-016, FT-017, FT-018 | customer-app, customer-web, api, worker | J-02: with two stores stocking an item, the customer searches, fills a one-store cart, sees fee and ETA, orders for another recipient and pays; also an address outside every zone, and a duplicate payment webhook | The nearer store is chosen; the order is confirmed with number, ETA and code; the recipient gets the SMS; the out-of-zone order is refused with the reason; the duplicate webhook changes nothing |
| TC-003 | FR-010, FR-011, FR-012 | FT-019, FT-020, FT-021, FT-022, FT-023, FT-024 | customer-app, customer-web, drivers-app, api, worker | J-03: the customer and the recipient follow a delivery to the door, then the customer rates it; separately, the customer cancels an order before the store accepts it | Map, status and notifications update at each step; the rating is saved; the cancelled order is refunded automatically |
| TC-004 | FR-013 | FT-025, FT-026, FT-027 | customer-app, customer-web, admin-web, api | J-04: the customer opens a ticket about an order, chats with support and receives a partial refund | Messages stay in one thread; the refund amount and status are shown |
| TC-005 | FR-014, FR-015 | FT-028, FT-029, FT-030, FT-031, FT-032, FT-098 | vendor-web, vendor-app, admin-web, api | J-05: a vendor applies, a support agent approves it, the owner signs in with an email code, adds a store and a staff login; also a declined application | The store is listed to nearby customers; the staff member sees only their store; the declined vendor sees the reason |
| TC-006 | FR-016, FR-017 | FT-033, FT-034, FT-035, FT-036, FT-037, FT-038 | vendor-web, vendor-app, customer-app, api | J-06: the owner adds products, imports a CSV with one bad row, marks an item out of stock, archives one and pauses the store | Customers see the changes at once; the bad row is listed with its reason; the paused store receives no orders |
| TC-007 | FR-006, FR-018, FR-019 | FT-039, FT-040, FT-041, FT-042, FT-043, FT-044 | vendor-web, vendor-app, api, worker | J-07: an order arrives and is accepted and marked ready; a second is declined; a third is left past `ORDER_ACCEPT_TIMEOUT` | Dispatch starts on accept and the driver is told when it's ready; declined and timed-out orders move to the next nearest store; earnings show sales and commission |
| TC-008 | FR-020 | FT-045, FT-046, FT-047, FT-048 | drivers-app, drivers-web, admin-web, api | J-08: a driver applies with documents, is declined with a reason, resubmits and is approved | The approved driver can sign in and go online; the declined driver saw the reason |
| TC-009 | FR-021, FR-022 | FT-049, FT-050, FT-051, FT-052, FT-053, FT-054, FT-055, FT-056, FT-057 | drivers-app, api, worker | J-09: two drivers are online; the nearer one declines, the other accepts, picks up and delivers with the right code; also a wrong code and a reported problem | The offer moves on after the decline; delivery completes only with the right code and credits the driver; the problem reaches support |
| TC-010 | FR-023 | FT-058, FT-059 | drivers-app, drivers-web, api, worker | J-10: a driver with completed deliveries checks earnings and requests a withdrawal | Amounts are correct and the withdrawal is in the next payout batch |
| TC-011 | FR-024, FR-025, FR-026 | FT-060, FT-061, FT-062, FT-063, FT-064, FT-065, FT-066, FT-067, FT-068, FT-069 | admin-web, admin-app, api, worker | J-11: an operations manager watches a late delivery and reassigns it; a support agent finds an order, refunds above and below `REFUND_AGENT_LIMIT`, cancels another; a driver and a store are suspended; an order gets no driver within `DISPATCH_ALERT_MINUTES` | Both drivers are notified of the reassignment; the refund above the limit needs a manager; suspended accounts can't work; the stuck-order alert appears; staff without the role can't do any of this |
| TC-012 | FR-027, FR-028, FR-029, FR-030, FR-037 | FT-070, FT-071, FT-072, FT-073, FT-074, FT-075, FT-076, FT-077, FT-099, FT-100, FT-102 | admin-web, admin-app, api | J-12: an admin creates a zone with pricing, invites a staff member, who signs in with a code and signs out, then suspends them; a finance officer must confirm sign-in with an authenticator app, creates a promo code; HR edits a staff record; the admin reviews the audit trail | New quotes use the zone prices; the staff member sees only their role's tools and the suspended one is signed out; finance sign-in without the second factor is refused; the promo applies at checkout; every change is in the audit trail |
| TC-013 | FR-031, FR-032 | FT-078, FT-079, FT-080, FT-081 | admin-web, admin-app, api, worker | J-13: a payout cycle ends; finance approves the batch with one failing transfer, exports a statement and checks reconciliation | The batch is net of commission; the failed transfer is listed; unmatched payments are flagged; the export downloads |
| TC-014 | FR-032 | FT-082 | admin-web, admin-app, api | J-14: management opens the dashboard for a zone and period | Figures match the orders and deliveries of that period |
| TC-015 | FR-033, FR-034 | FT-083, FT-084, FT-085, FT-086, FT-087, FT-088, FT-089, FT-090, FT-091 | partners-api, admin-web, api, worker | J-15: an admin creates a partner; with a live key the partner gets a quote, subscribes a webhook, books, tracks and cancels a delivery, and places a store order; also an invalid key | Each call succeeds and signed webhooks arrive for every status change; the invalid key is refused |
| TC-016 | FR-035 | FT-092, FT-093, FT-094, FT-095, FT-096 | mcp-server, customer-app, api | J-16: a customer connects an agent with limited scopes; the agent searches, places an order and tracks it; a staff-connected agent lists stuck orders | The order is placed only after the customer confirms in the app; out-of-scope calls are refused; the staff agent can't change anything |

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
