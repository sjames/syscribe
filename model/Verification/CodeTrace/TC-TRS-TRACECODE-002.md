---
type: TestCase
id: TC-TRS-TRACECODE-002
name: "The trace-code baseline only shrinks and is written through the guarded write layer"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/trace_code.rs
verifies:
  - REQ-TRS-TRACECODE-002
tags:
  - cli
---

```gherkin
Feature: shrink-only baseline

  Scenario: the baseline only shrinks
    Given a baseline written with --write-baseline
    Then trace-code exits 0
    When a new crate appears it exits 2
    When a baselined unit becomes traced or disappears it exits 2 with the entry reported stale
    And --write-baseline keeps existing reasons, drops stale entries and adds new ones as TODO

  Scenario: write-baseline is a guarded write
    Given protected_paths covers the baseline file
    Then an agent's --write-baseline is refused with exit 1 and a person's succeeds
```
