---
id: TC-TRS-NEXT-001
type: TestCase
testLevel: L3
status: active
name: "Verify next offers unblocked, unclaimed leaf items with hazards and the reason for each skip."
verifies:
  - REQ-TRS-NEXT-001
sourceFile: repo:crates/syscribe/tests/next_done.rs
testFunctions:
  - next_offers_unblocked_unclaimed_leaves_resumable_first
  - next_reports_hazards_and_exits_2_when_nothing_is_offered
---

```gherkin
Feature: next (TC-TRS-NEXT-001)

  Scenario: The behaviour of REQ-TRS-NEXT-001 holds
    Given a planning board built to exercise it
    When the command runs
    Then the result is as the requirement states
```
