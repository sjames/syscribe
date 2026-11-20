---
id: TC-TRS-DONE-001
type: TestCase
testLevel: L3
status: active
name: "Verify done names exactly what is missing, never writes, and reports a handoff under human-only status."
verifies:
  - REQ-TRS-DONE-001
sourceFile: repo:crates/syscribe/tests/next_done.rs
testFunctions:
  - done_names_exactly_what_is_missing
  - done_passes_when_everything_holds_and_never_writes
  - done_judges_a_composite_by_its_steps
  - done_reports_a_handoff_when_the_policy_reserves_done_for_a_person
  - done_rejects_an_unknown_item_and_a_non_planning_item
---

```gherkin
Feature: done (TC-TRS-DONE-001)

  Scenario: The behaviour of REQ-TRS-DONE-001 holds
    Given a planning board built to exercise it
    When the command runs
    Then the result is as the requirement states
```
