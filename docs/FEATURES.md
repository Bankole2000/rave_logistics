# Features

Status: Draft | Owner: @bankole2000 | Version: 0.3 | Last updated: 2026-11-26
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
| U-01 | Customer | person | Someone in a served Nigerian state who orders food, groceries and similar items from nearby restaurants and shops, for themselves or for someone else. | Must |
| U-02 | Recipient | person | The person an order is delivered to when it isn't the customer. Needs no account; follows the order from an SMS link. | Should |
| U-03 | Driver | person | An approved independent rider or driver (bike, car or van) who picks up orders from stores and delivers them. | Must |
| U-04 | Vendor owner | person | Owner or manager of a restaurant or shop; registers the business, sets up stores and products, and receives payouts. | Must |
| U-05 | Vendor staff | person | Works in a vendor's store; handles incoming orders and stock during opening hours. | Must |
| U-06 | Support agent | person | RaveConnect operations staff: answers customers, drivers and vendors, fixes orders, and checks driver and vendor applications. | Must |
| U-07 | Operations manager | person | RaveConnect operations lead: watches live deliveries by zone, reassigns drivers and suspends drivers or stores. | Must |
| U-08 | Admin | person | Configures the platform: service zones, pricing, staff accounts and partners. | Must |
| U-09 | HR officer | person | Keeps internal staff records. | Could |
| U-10 | Finance officer | person | Reconciles payments, approves vendor and driver payouts and exports financial reports. | Must |
| U-11 | Management | person | Company leadership; reads performance reports. | Should |
| U-12 | Partner app | app | An e-commerce partner's system that books deliveries or places orders through the Partner API. | Should |
| U-13 | AI agent | agent | An AI assistant or automated flow acting for a customer or staff member through the MCP server, within the scopes they granted. | Could |
| U-14 | System | system | The product itself: checks on every request, calculations, store sourcing, driver dispatch and scheduled jobs. | Must |
| U-15 | Payment provider | app | The payment gateway (vendor TBD, see Gaps) that confirms payments and transfers through webhooks. | Must |
| U-16 | Partner developer | person | A developer at an e-commerce partner who reads the API documentation and builds the integration. | Should |

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
| A-12 | places | Submits a cart or request as an order. | yes |
| A-13 | pays | Pays for an order through the payment provider. | yes |
| A-14 | accepts | Takes on an order or delivery offered to them. | yes |
| A-15 | declines | Refuses an order, delivery offer or application, with a reason. | yes |
| A-16 | assigns | Gives a delivery to a driver. | yes |
| A-17 | picks up | Collects an order from a store. | yes |
| A-18 | delivers | Hands an order to its recipient and confirms it. | yes |
| A-19 | cancels | Stops an order or delivery before it is completed. | yes |
| A-20 | rates | Scores and reviews a store or driver after an order. | yes |
| A-21 | tracks | Follows the live status and position of a delivery. | no |
| A-22 | approves | Accepts an application or a payout after checking it. | yes |
| A-23 | suspends | Blocks an account or store from working until lifted. | yes |
| A-24 | refunds | Returns all or part of a payment to the customer. | yes |
| A-25 | notifies | Sends a push notification, SMS, email or webhook. | yes |
| A-26 | signs up | Creates their own account or application. | yes |
| A-27 | exports | Downloads records as a file (CSV or PDF). | no |
| A-28 | calculates | Works out a derived value such as a fee, ETA or balance. | no |
| A-29 | sources | Chooses which store fulfils an order (nearest open store with the items in stock). | yes |

## Resources
Kinds: `stored` (kept in the database) or `derived` (calculated from stored data, e.g. a report or a total).

| ID | Name | Kind | Belongs to | Description |
|---|---|---|---|---|
| R-01 | Customer account | stored | — | Name, phone number (verified), email, saved payment references. |
| R-02 | Address | stored | R-01 | Map pin, street, landmark notes, state and contact phone. |
| R-03 | Vendor | stored | — | The business: name, category, CAC registration, bank account, commission rate, status (applied, approved, declined, suspended). |
| R-04 | Store | stored | R-03 | One branch: location, opening hours, delivery radius, prep time, open or paused. |
| R-05 | Product | stored | R-04 | Name, photo, price, options (size, extras), category. |
| R-06 | Stock level | stored | R-05 | Whether, and how many of, a product a store has available now. |
| R-07 | Cart | stored | R-01 | Items from one store waiting to be ordered. |
| R-08 | Order | stored | R-01 | Items, store, recipient, drop-off address, totals, delivery code and status. |
| R-09 | Delivery | stored | R-08 | One trip: pickup, drop-off, driver, earnings, status; also created directly by partners. |
| R-10 | Driver location | stored | R-11 | GPS positions of an online driver, kept for a limited period (setting `LOCATION_RETENTION_DAYS`, default 30). |
| R-11 | Driver | stored | — | Profile, NIN, licence, vehicle, guarantor, bank account, rating, status (applied, approved, declined, suspended) and availability. |
| R-12 | Payment | stored | R-08 | A charge through the payment provider and its status. |
| R-13 | Refund | stored | R-12 | A full or partial return of a payment, with reason and status. |
| R-14 | Rating | stored | R-08 | Score and comment for a store or driver. |
| R-15 | Support ticket | stored | — | An issue raised by a customer, driver or vendor, usually about an order; assignee and status. |
| R-16 | Message | stored | R-15 | One message in a ticket or order chat. |
| R-17 | Notification | stored | — | A push, SMS, email or webhook sent, with delivery status. |
| R-18 | Earnings balance | derived | — | A driver's or vendor's earned, pending and available amounts. |
| R-19 | Payout | stored | — | A transfer of earnings to a driver's or vendor's bank account. |
| R-20 | Delivery quote | derived | — | Delivery fee and ETA for a pickup and drop-off, from distance and zone pricing. |
| R-21 | Service zone | stored | — | An area inside one state where RaveConnect operates, with its pricing and commission. |
| R-22 | Staff member | stored | — | An internal employee: role (support, operations, finance, HR, admin, management), HR details, status. |
| R-23 | Vendor team member | stored | R-03 | A vendor's staff login and the stores they work in. |
| R-24 | Partner | stored | — | An e-commerce partner and its API keys (sandbox and live). |
| R-25 | Webhook subscription | stored | R-24 | A partner URL and the events it receives. |
| R-26 | API documentation | stored | — | Public reference and guides for the Partner API, with a sandbox. |
| R-27 | Report | derived | — | Totals and trends: orders, sales, delivery times, payouts, by zone and period. |
| R-28 | Audit entry | stored | — | Who did what to which record, and when, for every staff and partner write. |
| R-29 | Promotion | stored | — | A promo code or discount with its rules and limits. |
| R-30 | Session | stored | — | A signed-in device of any person. |
| R-31 | Agent connection | stored | — | A user's grant letting an AI agent act for them through MCP, with scopes and expiry. |

## Surfaces
Where a feature is offered: one row per app in `apps/` (from the project setup), plus SDKs or other packages people use directly.

| ID | Name | App or package | Description | Priority |
|---|---|---|---|---|
| S-01 | Customer mobile app | apps/customer-app | Android and iOS app for ordering and tracking. | Must |
| S-02 | Customer website | apps/customer-web | Ordering website; also hosts the recipient tracking page. | Should |
| S-03 | Driver mobile app | apps/drivers-app | Android and iOS app for going online, taking and completing deliveries. | Must |
| S-04 | Driver website | apps/drivers-web | Sign-up, documents and earnings for drivers. | Could |
| S-05 | Operations website | — | Deprecated: operations tools are part of the admin website (S-11), by role. | Won't |
| S-06 | Operations desktop app | — | Deprecated: operations tools are part of the admin app (S-13), by role. | Won't |
| S-07 | Operations mobile app | — | Deprecated: operations tools are part of the admin app (S-13), by role. | Won't |
| S-08 | Vendor website | apps/vendor-web | Store setup, catalogue, stock, orders and earnings for vendors. | Must |
| S-09 | Vendor desktop app | — | Deprecated: the vendor app (S-10) covers desktop. | Won't |
| S-10 | Vendor app | apps/vendor-app | Mobile (Android, iOS) and desktop app: orders and stock from a phone, tablet or store counter, with alerts. | Should |
| S-11 | Admin website | apps/admin-web | Back office for every staff role: support, operations (approvals, live tracking), admin (zones, pricing, staff, partners), finance, HR and management. Each role sees only its tools. | Must |
| S-12 | Admin desktop app | — | Deprecated: the admin app (S-13) covers desktop. | Won't |
| S-13 | Admin app | apps/admin-app | Mobile (Android, iOS) and desktop app with the back-office tools of S-11, by role. | Could |
| S-14 | Partner API | apps/partners-api | Public REST API for e-commerce partners, with webhooks. | Should |
| S-15 | Developer docs | apps/partners-api | Public documentation site for the Partner API, served by the partners-api app. | Should |
| S-16 | MCP server | apps/mcp-server | Model Context Protocol tools for AI agents acting for customers or staff. | Could |
| S-17 | Core API | apps/api | Backend for every client app and for partners-api and mcp-server; receives payment provider webhooks. | Must |
| S-18 | Worker | apps/worker | Background jobs: store sourcing and driver offer timeouts, notifications, payout calculation and partner webhook delivery. | Must |

## Journeys
A journey is what one user does to reach a goal, as features in order. The Milestone column is generated from ROADMAP.md, where each milestone lists the journeys it delivers.

| ID | Name | Main user | Goal | Features, in order | Milestone |
|---|---|---|---|---|---|
| J-01 | Get started as a customer | U-01 | Have an account and a delivery address | FT-001, FT-002, FT-003, FT-004, FT-005, FT-097 | M2 |
| J-02 | Order from a nearby store | U-01 | Place and pay for an order fulfilled by the nearest store | FT-006, FT-007, FT-008, FT-009, FT-010, FT-011, FT-012, FT-013, FT-014, FT-015, FT-016, FT-017, FT-018 | M2 |
| J-03 | Receive an order | U-01 | Follow the order and get it delivered | FT-019, FT-020, FT-021, FT-022, FT-023, FT-024 | M2 |
| J-04 | Get help with an order | U-01 | Have a problem resolved or money refunded | FT-025, FT-026, FT-027 | M3 |
| J-05 | Join as a vendor | U-04 | Have an approved business with a live store | FT-028, FT-029, FT-030, FT-031, FT-032 | M1 |
| J-06 | Manage the catalogue | U-04 | Customers see the right products, prices and stock | FT-033, FT-034, FT-035, FT-036, FT-037, FT-038 | M1 |
| J-07 | Fulfil an order as a vendor | U-05 | Prepare orders and hand them to drivers | FT-039, FT-040, FT-041, FT-042, FT-043, FT-044 | M2 |
| J-08 | Join as a driver | U-03 | Be approved to take deliveries | FT-045, FT-046, FT-047, FT-048 | M1 |
| J-09 | Deliver an order | U-03 | Pick up and deliver orders | FT-049, FT-050, FT-051, FT-052, FT-053, FT-054, FT-055, FT-056, FT-057 | M2 |
| J-10 | Get paid as a driver | U-03 | See and withdraw earnings | FT-058, FT-059 | M3 |
| J-11 | Run daily operations | U-07 | Keep every order moving and every complaint answered | FT-060, FT-061, FT-062, FT-063, FT-064, FT-065, FT-066, FT-067, FT-068, FT-069 | M3 |
| J-12 | Configure the platform | U-08 | Zones, pricing and staff are set up | FT-070, FT-071, FT-072, FT-073, FT-074, FT-075, FT-076, FT-077 | M1 |
| J-13 | Pay out vendors and drivers | U-10 | Everyone is paid correctly and on time | FT-078, FT-079, FT-080, FT-081 | M3 |
| J-14 | Review performance | U-11 | Know how the business is doing | FT-082 | M3 |
| J-15 | Integrate as a partner | U-12 | Book and follow deliveries automatically | FT-083, FT-084, FT-085, FT-086, FT-087, FT-088, FT-089, FT-090, FT-091 | M4 |
| J-16 | Automate with an AI agent | U-13 | An agent orders, tracks or triages on someone's behalf | FT-096, FT-092, FT-093, FT-094, FT-095 | M6 |

## Features
| ID | User | Action | Resource | Scope | Surfaces | Outcome | Priority | Depends on | Journey | Requirements | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|
| FT-001 | U-01 Customer | A-26 signs up | R-01 Customer account | — | S-01, S-02 | Account is created after the phone number is verified with a one-time code. | Must | — | J-01 | FR-001 | approved |
| FT-002 | U-01 Customer | A-08 signs in | R-30 Session | own account | S-01, S-02 | Signed in with phone number and one-time code; stays signed in on that device. | Must | FT-001 | J-01 | FR-001 | approved |
| FT-003 | U-01 Customer | A-01 creates | R-02 Address | own account | S-01, S-02 | Address is saved with a map pin and landmark notes and can be picked at checkout. | Must | FT-001 | J-01 | FR-002 | approved |
| FT-004 | U-01 Customer | A-03 edits | R-02 Address | own addresses | S-01, S-02 | Changed address is used for new orders; past orders keep the old one. | Should | FT-003 | J-01 | FR-002 | approved |
| FT-005 | U-01 Customer | A-09 signs out | R-30 Session | own sessions | S-01, S-02 | Device is signed out and needs a new code to sign in. | Must | FT-002 | J-01 | FR-001 | approved |
| FT-006 | U-01 Customer | A-02 views | R-04 Store | open stores that deliver to the chosen address | S-01, S-02 | Nearby open stores are listed nearest first, each with ETA and delivery fee. | Must | FT-003 | J-02 | FR-004 | approved |
| FT-007 | U-01 Customer | A-06 searches | R-05 Product | in-stock products of nearby open stores | S-01, S-02 | Matching products are listed with store, price and ETA, nearest store first. | Must | FT-006 | J-02 | FR-004 | approved |
| FT-008 | U-01 Customer | A-07 filters | R-04 Store | by category (food, groceries, pharmacy, …) | S-01, S-02 | Only stores of the chosen category are listed. | Should | FT-006 | J-02 | FR-004 | approved |
| FT-009 | U-01 Customer | A-02 views | R-05 Product | one store's catalogue | S-01, S-02 | Products are shown with photo, price, options and availability. | Must | FT-006 | J-02 | FR-004 | approved |
| FT-010 | U-01 Customer | A-03 edits | R-07 Cart | own cart; items from one store | S-01, S-02 | Items are added or removed and the total updates. | Must | FT-009 | J-02 | FR-005 | approved |
| FT-011 | U-14 System | A-28 calculates | R-20 Delivery quote | store to drop-off address, zone pricing | S-01, S-02, S-14 | Delivery fee and ETA are shown before payment. | Must | FT-070 | J-02 | FR-005 | approved |
| FT-012 | U-14 System | A-29 sources | R-08 Order | nearest open store with every item in stock | S-01, S-02 | The customer sees which store will fulfil the order; if it declines or times out, the order moves to the next nearest store (asking the customer if the price changes) or is refunded. | Must | FT-035 | J-02 | FR-006 | approved |
| FT-013 | U-01 Customer | A-12 places | R-08 Order | own cart, to own address | S-01, S-02 | Order is confirmed with a number, ETA and delivery code. | Must | FT-010, FT-011 | J-02 | FR-005 | approved |
| FT-014 | U-01 Customer | A-12 places | R-08 Order | for another recipient (name, phone, address) | S-01, S-02 | Recipient gets an SMS with a tracking link and the delivery code. | Should | FT-013 | J-02 | FR-007 | approved |
| FT-015 | U-01 Customer | A-13 pays | R-12 Payment | own order; card, bank transfer or USSD | S-01, S-02 | Payment is confirmed and the order is sent to the store. | Must | FT-013 | J-02 | FR-008 | approved |
| FT-016 | U-15 Payment provider | A-11 records | R-12 Payment | signed webhooks only | S-17 | Payment status updates automatically; duplicate events change nothing. | Must | FT-015 | J-02 | FR-008 | approved |
| FT-017 | U-14 System | A-10 rejects | R-08 Order | drop-off outside a service zone or the store's state, store closed, item out of stock | S-01, S-02, S-14 | Customer sees why the order can't be placed. | Must | FT-070 | J-02 | FR-005 | approved |
| FT-018 | U-01 Customer | A-12 places | R-08 Order | again from a past order | S-01, S-02 | Cart is filled with the items still available and goes to checkout. | Could | FT-023 | J-02 | FR-009 | approved |
| FT-019 | U-01 Customer | A-21 tracks | R-09 Delivery | own orders | S-01, S-02 | Live status and the driver's position on a map with an updated ETA. | Must | FT-053 | J-03 | FR-010 | approved |
| FT-020 | U-02 Recipient | A-21 tracks | R-09 Delivery | from the SMS link, no account | S-02 | Recipient sees status, ETA and the driver on a map. | Should | FT-014 | J-03 | FR-007 | approved |
| FT-021 | U-14 System | A-25 notifies | R-17 Notification | customer and recipient, on order status changes | S-01, S-02 | Push or SMS at accepted, picked up, arriving and delivered. | Must | FT-013 | J-03 | FR-010 | approved |
| FT-022 | U-01 Customer | A-19 cancels | R-08 Order | own order, before the store accepts it | S-01, S-02 | Order is cancelled and the refund starts automatically. | Must | FT-013 | J-03 | FR-011 | approved |
| FT-023 | U-01 Customer | A-02 views | R-08 Order | own order history | S-01, S-02 | Past and current orders with items, totals and receipts. | Should | FT-013 | J-03 | FR-011 | approved |
| FT-024 | U-01 Customer | A-20 rates | R-14 Rating | own delivered orders; store and driver | S-01, S-02 | Rating is saved and counts towards the store's and driver's scores. | Should | FT-055 | J-03 | FR-012 | approved |
| FT-025 | U-01 Customer | A-01 creates | R-15 Support ticket | about own orders or account | S-01, S-02 | Ticket is opened and the customer can chat with support. | Must | FT-013 | J-04 | FR-013 | approved |
| FT-026 | U-01 Customer | A-01 creates | R-16 Message | in own tickets, or to the driver of own active order | S-01, S-02 | Message is delivered and the reply appears in the same thread. | Should | FT-025 | J-04 | FR-013 | approved |
| FT-027 | U-01 Customer | A-02 views | R-13 Refund | own refunds | S-01, S-02 | Refund amount and status are shown. | Should | FT-065 | J-04 | FR-013 | approved |
| FT-028 | U-04 Vendor owner | A-26 signs up | R-03 Vendor | own business; CAC registration, bank account, first store | S-08, S-10 | Application is submitted and its status is shown. | Must | — | J-05 | FR-014 | approved |
| FT-029 | U-06 Support agent | A-22 approves | R-03 Vendor | applications awaiting review | S-11, S-13 | Vendor is approved and notified; its stores can go live. A declined vendor sees the reason. | Must | FT-028 | J-05 | FR-014 | approved |
| FT-030 | U-04 Vendor owner | A-01 creates | R-04 Store | own business; one or more branches | S-08, S-10 | Store with location, hours, delivery radius and prep time is listed to nearby customers. | Must | FT-029 | J-05 | FR-014 | approved |
| FT-031 | U-04 Vendor owner | A-01 creates | R-23 Vendor team member | own business and stores | S-08, S-10 | Staff member gets a login limited to the chosen stores. | Should | FT-030 | J-05 | FR-015 | approved |
| FT-032 | U-05 Vendor staff | A-08 signs in | R-30 Session | own login | S-08, S-10 | Staff sees only the stores they work in. | Must | FT-031 | J-05 | FR-015 | approved |
| FT-033 | U-04 Vendor owner | A-01 creates | R-05 Product | own stores | S-08, S-10 | Product with photo, price and options appears in the store. | Must | FT-030 | J-06 | FR-016 | approved |
| FT-034 | U-04 Vendor owner | A-03 edits | R-05 Product | own stores | S-08, S-10 | New price or details show for new orders. | Must | FT-033 | J-06 | FR-016 | approved |
| FT-035 | U-05 Vendor staff | A-03 edits | R-06 Stock level | own stores | S-08, S-10 | Item is marked in or out of stock and customers stop or start seeing it. | Must | FT-033 | J-06 | FR-016 | approved |
| FT-036 | U-04 Vendor owner | A-01 creates | R-05 Product | many at once from a CSV file | S-08, S-10 | Products are imported; rows with errors are listed with the reason. | Should | FT-033 | J-06 | FR-017 | approved |
| FT-037 | U-04 Vendor owner | A-04 archives | R-05 Product | own stores | S-08, S-10 | Product disappears from the store and can be restored. | Should | FT-033 | J-06 | FR-017 | approved |
| FT-038 | U-05 Vendor staff | A-03 edits | R-04 Store | own stores; open or paused | S-08, S-10 | A paused store stops receiving orders until reopened. | Must | FT-030 | J-06 | FR-016 | approved |
| FT-039 | U-05 Vendor staff | A-02 views | R-08 Order | incoming orders of own stores | S-08, S-10 | New orders appear at once with a sound alert. | Must | FT-015 | J-07 | FR-018 | approved |
| FT-040 | U-05 Vendor staff | A-14 accepts | R-08 Order | own stores; with prep time | S-08, S-10 | Customer sees the order accepted and driver dispatch starts. | Must | FT-039 | J-07 | FR-018 | approved |
| FT-041 | U-05 Vendor staff | A-15 declines | R-08 Order | own stores; with a reason | S-08, S-10 | Order moves to the next nearest store or is refunded. | Must | FT-039 | J-07 | FR-018 | approved |
| FT-042 | U-14 System | A-29 sources | R-08 Order | not accepted within `ORDER_ACCEPT_TIMEOUT` (default 3 minutes) | S-08 | Order moves to the next nearest store and the first store is told it timed out. | Must | FT-012 | J-07 | FR-006 | approved |
| FT-043 | U-05 Vendor staff | A-03 edits | R-08 Order | own stores; ready for pickup | S-08, S-10 | Assigned driver is told the order is ready. | Must | FT-040 | J-07 | FR-018 | approved |
| FT-044 | U-04 Vendor owner | A-02 views | R-18 Earnings balance | own business | S-08, S-10 | Sales, commission and the amount due at the next payout. | Must | FT-040 | J-07 | FR-019 | approved |
| FT-045 | U-03 Driver | A-26 signs up | R-11 Driver | own application; NIN, licence, vehicle, guarantor, bank account, photo | S-03, S-04 | Application is submitted and its status is shown. | Must | — | J-08 | FR-020 | approved |
| FT-046 | U-06 Support agent | A-22 approves | R-11 Driver | applications whose documents were checked | S-11, S-13 | Driver is approved and notified and can go online. | Must | FT-045 | J-08 | FR-020 | approved |
| FT-047 | U-06 Support agent | A-15 declines | R-11 Driver | applications awaiting review; with a reason | S-11, S-13 | Driver sees the reason and can fix and resubmit. | Should | FT-045 | J-08 | FR-020 | approved |
| FT-048 | U-03 Driver | A-08 signs in | R-30 Session | own account | S-03, S-04 | Signed in with phone number and one-time code. | Must | FT-045 | J-08 | FR-020 | approved |
| FT-049 | U-03 Driver | A-03 edits | R-11 Driver | own availability; online or offline | S-03 | An online driver shares location and receives delivery offers. | Must | FT-046 | J-09 | FR-021 | approved |
| FT-050 | U-14 System | A-16 assigns | R-09 Delivery | nearest available online driver to the store | S-03 | Driver gets an offer with pickup, drop-off and earnings; if declined or unanswered within `DRIVER_OFFER_TIMEOUT` (default 30 seconds), the next nearest driver gets it. | Must | FT-040, FT-049 | J-09 | FR-021 | approved |
| FT-051 | U-03 Driver | A-14 accepts | R-09 Delivery | offers sent to them | S-03 | Driver is guided to the store; customer sees the driver. | Must | FT-050 | J-09 | FR-021 | approved |
| FT-052 | U-03 Driver | A-15 declines | R-09 Delivery | offers sent to them | S-03 | Offer goes to the next driver. | Must | FT-050 | J-09 | FR-021 | approved |
| FT-053 | U-03 Driver | A-11 records | R-10 Driver location | while online | S-03 | Position updates every `LOCATION_INTERVAL_SECONDS` (default 10) for dispatch and tracking. | Must | FT-049 | J-09 | FR-021 | approved |
| FT-054 | U-03 Driver | A-17 picks up | R-09 Delivery | own deliveries; at the store | S-03 | Customer is told the order is on its way. | Must | FT-051 | J-09 | FR-022 | approved |
| FT-055 | U-03 Driver | A-18 delivers | R-09 Delivery | own deliveries; with the recipient's delivery code | S-03 | Delivery is completed and the driver's earnings are credited. | Must | FT-054 | J-09 | FR-022 | approved |
| FT-056 | U-03 Driver | A-03 edits | R-09 Delivery | own deliveries; report a problem | S-03 | Problem (recipient unreachable, wrong items, accident) reaches support at once. | Should | FT-051 | J-09 | FR-022 | approved |
| FT-057 | U-03 Driver | A-02 views | R-09 Delivery | own delivery history | S-03, S-04 | Past deliveries with earnings per trip. | Should | FT-055 | J-09 | FR-022 | approved |
| FT-058 | U-03 Driver | A-02 views | R-18 Earnings balance | own earnings | S-03, S-04 | Earned, pending and available amounts. | Must | FT-055 | J-10 | FR-023 | approved |
| FT-059 | U-03 Driver | A-01 creates | R-19 Payout | own available balance, to own bank account | S-03, S-04 | Withdrawal request is accepted and paid in the next payout run. | Should | FT-058 | J-10 | FR-023 | approved |
| FT-060 | U-07 Operations manager | A-21 tracks | R-09 Delivery | all active deliveries, by zone | S-11, S-13 | Live map and list of deliveries with late ones highlighted. | Must | FT-053 | J-11 | FR-024 | approved |
| FT-061 | U-07 Operations manager | A-16 assigns | R-09 Delivery | any active delivery | S-11, S-13 | Delivery moves to the chosen driver and both drivers are notified. | Must | FT-060 | J-11 | FR-024 | approved |
| FT-062 | U-06 Support agent | A-06 searches | R-08 Order | by order number, phone or name | S-11, S-13 | Matching orders with full status history. | Must | FT-013 | J-11 | FR-025 | approved |
| FT-063 | U-06 Support agent | A-03 edits | R-15 Support ticket | any ticket; assign, set status | S-11, S-13 | Ticket is assigned or resolved and the requester is notified. | Must | FT-025 | J-11 | FR-025 | approved |
| FT-064 | U-06 Support agent | A-01 creates | R-16 Message | in any ticket | S-11, S-13 | Reply reaches the customer, driver or vendor in their app. | Must | FT-063 | J-11 | FR-025 | approved |
| FT-065 | U-06 Support agent | A-24 refunds | R-13 Refund | full or partial, up to `REFUND_AGENT_LIMIT` (default ₦20,000) without a manager | S-11, S-13 | Refund is sent through the payment provider and the customer is notified. | Must | FT-015 | J-11 | FR-025 | approved |
| FT-066 | U-06 Support agent | A-19 cancels | R-08 Order | any order not yet delivered | S-11, S-13 | Order, delivery and store are updated and the refund rule applies. | Must | FT-062 | J-11 | FR-025 | approved |
| FT-067 | U-07 Operations manager | A-23 suspends | R-11 Driver | any driver; with a reason | S-11, S-13 | Driver goes offline and can't go online until reinstated. | Must | FT-046 | J-11 | FR-026 | approved |
| FT-068 | U-07 Operations manager | A-23 suspends | R-04 Store | any store; with a reason | S-11, S-13 | Store stops receiving orders until reinstated. | Should | FT-030 | J-11 | FR-026 | approved |
| FT-069 | U-14 System | A-25 notifies | R-17 Notification | operations staff, when an order has no driver within `DISPATCH_ALERT_MINUTES` (default 5) or is late | S-11, S-13 | Alert appears with the order and a link to act on it. | Should | FT-050 | J-11 | FR-024 | approved |
| FT-070 | U-08 Admin | A-01 creates | R-21 Service zone | an area inside one state | S-11, S-13 | Zone with its map area, pricing and commission is live for orders. | Must | — | J-12 | FR-027 | approved |
| FT-071 | U-08 Admin | A-03 edits | R-21 Service zone | any zone; pricing, commission, active | S-11, S-13 | New prices apply to new quotes. | Must | FT-070 | J-12 | FR-027 | approved |
| FT-072 | U-08 Admin | A-01 creates | R-22 Staff member | with a role | S-11, S-13 | Staff member gets an invitation and only the tools of their role. | Must | — | J-12 | FR-028 | approved |
| FT-073 | U-08 Admin | A-23 suspends | R-22 Staff member | any staff member | S-11, S-13 | Staff member is signed out everywhere and can't sign in. | Should | FT-072 | J-12 | FR-028 | approved |
| FT-074 | U-09 HR officer | A-03 edits | R-22 Staff member | HR details (job title, department, documents) | S-11, S-13 | Staff record is updated. | Could | FT-072 | J-12 | FR-028 | approved |
| FT-075 | U-08 Admin | A-02 views | R-28 Audit entry | all, filtered by person, record or date | S-11, S-13 | Who changed what and when. | Should | FT-076 | J-12 | FR-029 | approved |
| FT-076 | U-14 System | A-11 records | R-28 Audit entry | every staff and partner write | S-11, S-14 | An audit entry exists for each change. | Must | FT-072 | J-12 | FR-029 | approved |
| FT-077 | U-08 Admin | A-01 creates | R-29 Promotion | by zone, store or customer group | S-11, S-13 | Promo code applies at checkout within its rules. | Could | FT-070 | J-12 | FR-030 | approved |
| FT-078 | U-14 System | A-28 calculates | R-19 Payout | vendors and drivers, every `PAYOUT_CYCLE` (default weekly), minus commission | S-11 | A payout batch is ready for finance to approve. | Must | FT-055 | J-13 | FR-031 | approved |
| FT-079 | U-10 Finance officer | A-22 approves | R-19 Payout | a payout batch | S-11, S-13 | Transfers are sent through the payment provider; failures are listed. | Must | FT-078 | J-13 | FR-031 | approved |
| FT-080 | U-10 Finance officer | A-27 exports | R-27 Report | payments, refunds, commission and payouts by period | S-11, S-13 | CSV or PDF statement downloads. | Should | FT-079 | J-13 | FR-032 | approved |
| FT-081 | U-10 Finance officer | A-02 views | R-12 Payment | all payments, with reconciliation status | S-11, S-13 | Payments that don't match the provider are flagged. | Must | FT-016 | J-13 | FR-031 | approved |
| FT-082 | U-11 Management | A-02 views | R-27 Report | orders, sales, delivery times and growth, by zone and period | S-11, S-13 | Dashboard of key figures with trends. | Should | FT-055 | J-14 | FR-032 | approved |
| FT-083 | U-16 Partner developer | A-02 views | R-26 API documentation | public | S-15 | Reference, guides and a sandbox to try requests. | Should | — | J-15 | FR-033 | approved |
| FT-084 | U-08 Admin | A-01 creates | R-24 Partner | with sandbox and live API keys | S-11 | Partner receives keys; live keys work only after approval. | Should | — | J-15 | FR-033 | approved |
| FT-085 | U-12 Partner app | A-02 views | R-20 Delivery quote | own API key | S-14 | Fee and ETA for a pickup and drop-off. | Should | FT-084, FT-011 | J-15 | FR-033 | approved |
| FT-086 | U-12 Partner app | A-01 creates | R-09 Delivery | from the partner's pickup point to its customer, inside one state | S-14 | Delivery is booked and dispatched to the nearest driver. | Should | FT-085, FT-050 | J-15 | FR-033 | approved |
| FT-087 | U-12 Partner app | A-12 places | R-08 Order | from RaveConnect stores, for the partner's customer | S-14 | Order is sourced and delivered like a customer order. | Could | FT-084, FT-012 | J-15 | FR-034 | approved |
| FT-088 | U-12 Partner app | A-01 creates | R-25 Webhook subscription | own partner account | S-14 | Partner URL receives the chosen events, signed. | Should | FT-084 | J-15 | FR-033 | approved |
| FT-089 | U-14 System | A-25 notifies | R-17 Notification | partner webhooks, on delivery status changes; retried for `WEBHOOK_RETRY_HOURS` (default 24) | S-14 | Partner system receives each status change. | Should | FT-088 | J-15 | FR-033 | approved |
| FT-090 | U-12 Partner app | A-19 cancels | R-09 Delivery | own deliveries, before pickup | S-14 | Delivery is cancelled and the driver is released. | Should | FT-086 | J-15 | FR-033 | approved |
| FT-091 | U-12 Partner app | A-21 tracks | R-09 Delivery | own deliveries | S-14 | Status, driver position and ETA. | Should | FT-086 | J-15 | FR-033 | approved |
| FT-092 | U-13 AI agent | A-06 searches | R-05 Product | for a customer who connected the agent | S-16 | In-stock products of nearby stores, as for the customer. | Could | FT-096, FT-007 | J-16 | FR-035 | approved |
| FT-093 | U-13 AI agent | A-12 places | R-08 Order | for a customer who connected the agent; the customer confirms before payment | S-16 | Order is placed only after the customer confirms it in the app. | Could | FT-092, FT-013 | J-16 | FR-035 | approved |
| FT-094 | U-13 AI agent | A-21 tracks | R-09 Delivery | orders of the connected customer | S-16 | Status and ETA in the agent's conversation. | Could | FT-096, FT-019 | J-16 | FR-035 | approved |
| FT-095 | U-13 AI agent | A-02 views | R-08 Order | for a staff member who connected it; within their role | S-16 | Agent lists stuck or late orders for triage; changes still need a person. | Could | FT-096, FT-062 | J-16 | FR-035 | approved |
| FT-096 | U-01 Customer | A-01 creates | R-31 Agent connection | own account; chosen scopes and expiry | S-01, S-02 | Agent can act within the scopes until the connection expires or is revoked. | Could | FT-002 | J-16 | FR-035 | approved |
| FT-097 | U-01 Customer | A-05 removes | R-01 Customer account | own account | S-01, S-02 | Account and personal data are deleted, except records the law requires keeping. | Must | FT-001 | J-01 | FR-003 | approved |

## Coverage matrix
Generated from the Features table (approved and proposed features only); don't edit it by hand. Each cell lists the actions a user can take on a resource. An empty cell is not an error, but it is worth a look: who can change or remove this resource?

<!-- matrix:start -->
| User \ Resource | Customer account | Address | Vendor | Store | Product | Stock level | Cart | Order | Delivery | Driver location | Driver | Payment | Refund | Rating | Support ticket | Message | Notification | Earnings balance | Payout | Delivery quote | Service zone | Staff member | Vendor team member | Partner | Webhook subscription | API documentation | Report | Audit entry | Promotion | Session | Agent connection |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| U-01 Customer | signs up, removes | creates, edits |  | views, filters | searches, views |  | edits | places, cancels, views | tracks |  |  | pays | views | rates | creates | creates |  |  |  |  |  |  |  |  |  |  |  |  |  | signs in, signs out | creates |
| U-02 Recipient |  |  |  |  |  |  |  |  | tracks |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| U-03 Driver |  |  |  |  |  |  |  |  | accepts, declines, picks up, delivers, edits, views | records | signs up, edits |  |  |  |  |  |  | views | creates |  |  |  |  |  |  |  |  |  |  | signs in |  |
| U-04 Vendor owner |  |  | signs up | creates | creates, edits, archives |  |  |  |  |  |  |  |  |  |  |  |  | views |  |  |  |  | creates |  |  |  |  |  |  |  |  |
| U-05 Vendor staff |  |  |  | edits |  | edits |  | views, accepts, declines, edits |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | signs in |  |
| U-06 Support agent |  |  | approves |  |  |  |  | searches, cancels |  |  | approves, declines |  | refunds |  | edits | creates |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| U-07 Operations manager |  |  |  | suspends |  |  |  |  | tracks, assigns |  | suspends |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| U-08 Admin |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | creates, edits | creates, suspends |  | creates |  |  |  | views | creates |  |  |
| U-09 HR officer |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | edits |  |  |  |  |  |  |  |  |  |
| U-10 Finance officer |  |  |  |  |  |  |  |  |  |  |  | views |  |  |  |  |  |  | approves |  |  |  |  |  |  |  | exports |  |  |  |  |
| U-11 Management |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | views |  |  |  |  |
| U-12 Partner app |  |  |  |  |  |  |  | places | creates, cancels, tracks |  |  |  |  |  |  |  |  |  |  | views |  |  |  |  | creates |  |  |  |  |  |  |
| U-13 AI agent |  |  |  |  | searches |  |  | places, views | tracks |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| U-14 System |  |  |  |  |  |  |  | sources, rejects | assigns |  |  |  |  |  |  |  | notifies |  | calculates | calculates |  |  |  |  |  |  |  | records |  |  |  |
| U-15 Payment provider |  |  |  |  |  |  |  |  |  |  |  | records |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| U-16 Partner developer |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | views |  |  |  |  |  |
<!-- matrix:end -->

## Gaps
Questions found while building the lists that the features don't answer yet. Each needs a decision: add a feature (as `proposed`), or note here that it is intentionally missing.

- **Number of apps.** Decided in the project setup (2026-11-26). The apps are customer-web, customer-app, drivers-web, drivers-app, admin-app, admin-web, api, worker, vendor-web, vendor-app, partners-api and mcp-server. The operations tools are part of admin-web and admin-app, by role. admin-app and vendor-app are each one mobile and desktop app. The developer docs are served by partners-api. S-05, S-06, S-07, S-09 and S-12 are deprecated.
- **Core backend.** All surfaces need a shared backend (e.g. `apps/api`), and the Partner API and MCP server sit on top of it. The backend is not a surface itself, but it must be added to the apps list in the project setup.
- **Vendor choices (open decisions):** the payment provider (e.g. Paystack, Flutterwave), the SMS and one-time-code provider, and the maps and routing provider.
- **Cash on delivery:** not included. Deferred (2026-11-26): management decision.
- **Multi-store orders:** a cart holds items from one store only, and each order comes from one store (confirmed by the owner, 2026-11-26).
- **Parcel delivery for customers:** customers can't send a package of their own from A to B. Only partners can (FT-086). Decide whether customers get this too.
- **Scheduled orders, tips and vendor promotions:** not included.
- **Vendor and driver support:** vendors and drivers can't open tickets themselves yet. Only customers can (FT-025).
- **HR scope:** only staff records (FT-074) are covered. Payroll and leave are assumed to be outside the product.
- **Data protection:** the Nigeria Data Protection Act affects retention periods, consent, and the account deletion in FT-097. This needs `review:privacy`.
- **Commission and fees:** these are assumed to be settings per zone and per vendor. Who sets a vendor's rate is not decided.

## Not features
Requirements that are not user + action + resource stay in the PRD: quality targets (NFR-), constraints, and "works on every surface" requirements, which the Surfaces list covers.

## Approval
{{Human approver, revision, date}}
