# Product Requirements Document

Status: Draft | Owner: @bankole2000 | Version: 0.2 | Last updated: 2026-11-26
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
| FR-001 | FT-001, FT-002, FT-005 | Customers sign up and sign in with a verified phone number, and sign out. | A new phone number gets an account after entering the one-time code; signing out ends the session on that device. | Must |
| FR-002 | FT-003, FT-004 | Customers save and change delivery addresses with a map pin and landmark notes. | A saved address can be picked at checkout; editing it doesn't change past orders. | Must |
| FR-003 | FT-097 | Customers delete their own account and personal data. | After deletion the customer can't sign in and their personal data is gone, except records the law requires keeping. | Must |
| FR-004 | FT-006, FT-007, FT-008, FT-009 | Customers find nearby open stores and in-stock products. | For a chosen address, only open stores that deliver there are listed, nearest first, with ETA and fee; search and category filters show in-stock items only. | Must |
| FR-005 | FT-010, FT-011, FT-013, FT-017 | Customers order from one store, seeing the delivery fee and ETA first, inside the service zones only. | An order from a one-store cart is confirmed with a number, ETA and delivery code; orders outside a zone or the store's state, from a closed store or with out-of-stock items are refused with the reason. | Must |
| FR-006 | FT-012, FT-042 | The system sources each order from the nearest open store that has every item in stock, and moves it on when that store declines or doesn't accept in time. | The customer sees the fulfilling store; a decline or a timeout (`ORDER_ACCEPT_TIMEOUT`, default 3 minutes) moves the order to the next nearest store, asking the customer if the price changes, or refunds it. | Must |
| FR-007 | FT-014, FT-020 | Customers send an order to another recipient, who follows it without an account. | The recipient gets an SMS with the delivery code and a tracking link showing status, ETA and the driver. | Should |
| FR-008 | FT-015, FT-016 | Customers pay online, and payment status comes only from verified payment provider webhooks. | A paid order reaches the store; unsigned or duplicate webhook events change nothing. | Must |
| FR-009 | FT-018 | Customers reorder a past order. | The cart is filled with the items still available. | Could |
| FR-010 | FT-019, FT-021 | Customers track deliveries live and are notified of status changes. | Map, status and ETA update while the driver moves; push or SMS at accepted, picked up, arriving and delivered. | Must |
| FR-011 | FT-022, FT-023 | Customers cancel an order before the store accepts it and see their order history. | A cancelled order starts its refund automatically; past orders show items, totals and receipts. | Must |
| FR-012 | FT-024 | Customers rate the store and driver of a delivered order. | The rating counts towards the store's and driver's scores. | Should |
| FR-013 | FT-025, FT-026, FT-027 | Customers get help with their orders and follow their refunds. | A ticket opens a chat with support; messages and replies stay in one thread; refund amount and status are visible. | Must |
| FR-014 | FT-028, FT-029, FT-030 | Vendors apply, are approved by support, and set up their stores. | An approved vendor's store, with location, hours, radius and prep time, is listed to nearby customers; a declined vendor sees the reason. | Must |
| FR-015 | FT-031, FT-032 | Vendor owners give their staff logins limited to chosen stores. | A staff member signs in and sees only their stores. | Must |
| FR-016 | FT-033, FT-034, FT-035, FT-038 | Vendors manage products, prices, stock and whether a store is open. | Changes show to customers at once; out-of-stock items and paused stores receive no orders. | Must |
| FR-017 | FT-036, FT-037 | Vendors import products from a CSV file and archive products. | Imported rows appear in the store and rows with errors are listed; archived products disappear and can be restored. | Should |
| FR-018 | FT-039, FT-040, FT-041, FT-043 | Vendor staff receive, accept or decline, and mark orders ready. | New orders alert at once; accepting starts dispatch; declining moves the order on; ready notifies the driver. | Must |
| FR-019 | FT-044 | Vendors see their earnings. | Sales, commission and the amount due at the next payout. | Must |
| FR-020 | FT-045, FT-046, FT-047, FT-048 | Drivers apply with their documents and are approved or declined by support. | An approved driver can sign in and go online; a declined driver sees the reason and can resubmit. | Must |
| FR-021 | FT-049, FT-050, FT-051, FT-052, FT-053 | The system offers each delivery to the nearest available online driver. | Online drivers share their location every `LOCATION_INTERVAL_SECONDS` (default 10); an offer declined or unanswered within `DRIVER_OFFER_TIMEOUT` (default 30 seconds) goes to the next nearest driver. | Must |
| FR-022 | FT-054, FT-055, FT-056, FT-057 | Drivers pick up and deliver orders, confirming delivery with the recipient's code, and report problems. | Delivery completes only with the right code and credits the driver; reported problems reach support at once. | Must |
| FR-023 | FT-058, FT-059 | Drivers see their earnings and withdraw their available balance. | Earned, pending and available amounts; a withdrawal is paid in the next payout run. | Must |
| FR-024 | FT-060, FT-061, FT-069 | Operations staff watch all active deliveries, reassign drivers and are alerted to stuck orders. | Live map by zone with late deliveries highlighted; reassignment notifies both drivers; orders without a driver after `DISPATCH_ALERT_MINUTES` (default 5) raise an alert. | Must |
| FR-025 | FT-062, FT-063, FT-064, FT-065, FT-066 | Support agents find orders, answer tickets, cancel orders and refund customers. | Refunds up to `REFUND_AGENT_LIMIT` (default ₦20,000) need no manager; every action notifies the person affected. | Must |
| FR-026 | FT-067, FT-068 | Operations managers suspend drivers and stores. | A suspended driver can't go online and a suspended store receives no orders until reinstated. | Must |
| FR-027 | FT-070, FT-071 | Admins define service zones inside a state, with pricing and commission. | New zones and prices apply to new quotes. | Must |
| FR-028 | FT-072, FT-073, FT-074 | Admins manage staff accounts and roles; HR keeps staff records. | Each staff member sees only their role's tools; a suspended staff member is signed out everywhere. | Must |
| FR-029 | FT-075, FT-076 | Every staff and partner write is audited. | An admin can see who changed what and when, filtered by person, record or date. | Must |
| FR-030 | FT-077 | Admins create promo codes. | A promo code applies at checkout within its rules. | Could |
| FR-031 | FT-078, FT-079, FT-081 | Finance reconciles payments and approves vendor and driver payouts. | Each `PAYOUT_CYCLE` (default weekly) produces a payout batch net of commission; approved transfers are sent and failures listed; unmatched payments are flagged. | Must |
| FR-032 | FT-080, FT-082 | Finance and management get reports. | Statements download as CSV or PDF; a dashboard shows orders, sales, delivery times and growth by zone and period. | Should |
| FR-033 | FT-083, FT-084, FT-085, FT-086, FT-088, FT-089, FT-090, FT-091 | E-commerce partners book, follow and cancel deliveries through a documented API with webhooks. | With a live key, a partner gets a quote, books a delivery that is dispatched to the nearest driver, and receives signed webhook events, retried for `WEBHOOK_RETRY_HOURS` (default 24). | Should |
| FR-034 | FT-087 | Partners place orders from RaveConnect stores for their customers. | The order is sourced and delivered like a customer order. | Could |
| FR-035 | FT-092, FT-093, FT-094, FT-095, FT-096 | Customers and staff let AI agents act for them through MCP, within granted scopes. | An agent can search, track and triage within its scopes; it places an order only after the customer confirms it, and staff changes still need a person. | Could |

## Out of scope / stretch
- Cash on delivery: deferred, pending a management decision (FEATURES.md, Gaps).
- Orders that combine items from several stores: each order comes from one store.
- Parcel delivery booked by customers (only partners book deliveries directly, FR-033), scheduled orders and tips.
- HR beyond staff records (payroll, leave).

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
- Payment provider (e.g. Paystack, Flutterwave), SMS and one-time-code provider, and maps and routing provider. Owner: @bankole2000.
- Who sets each vendor's commission rate. Owner: @bankole2000.
- Data protection under the Nigeria Data Protection Act (retention, consent, account deletion): needs `review:privacy`.

## Approval
{{Human approver, date, approved revision}}
