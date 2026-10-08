---
id: TC-TRS-VIS-027
type: TestCase
testLevel: L3
status: active
name: "Verify the live planning dashboard: status counts, working-now grouping by agent and person, columns, filters, escaping, empty state and refresh on a changed claim."
verifies:
  - REQ-TRS-VIS-027
sourceFile: repo:crates/syscribe-server/tests/planning_dashboard.rs
testFunctions:
  - the_page_is_served_and_wires_live_refresh
  - the_board_counts_every_status_and_groups_who_is_working
  - done_cards_appear_on_request
  - filtering_by_who_keeps_only_that_persons_or_agents_items
  - user_controlled_text_is_escaped
  - a_model_without_planning_items_shows_an_empty_state
  - a_changed_claim_shows_on_the_next_board_fetch
tags:
  - planning
  - dashboard
---

Hosted in `crates/syscribe-server/tests/planning_dashboard.rs`
(`cargo test -p syscribe-server --test planning_dashboard`); the client helpers are checked by
`frontend/test/planning-core.test.mjs` (`npm test` in `crates/syscribe-server/frontend/`).

```gherkin
Feature: planning dashboard (TC-TRS-VIS-027)

  Scenario: the board
    Then it counts every status and groups agent, person and unassigned work, naming what blocked cards wait on and hiding done cards but counting them

  Scenario: filters, safety, empty
    Then who and done filter it, markup in names is escaped and an empty model says so

  Scenario: live
    When a claim changes in the model
    Then the next board fetch shows it
```
