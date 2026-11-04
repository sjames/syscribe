---
type: TestCase
id: TC-TRS-BIND-001
name: "bind previews, refuses what the model's checks refuse, and edits only the affected lines"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/bind_timeline.rs
verifies:
  - REQ-TRS-BIND-001
tags:
  - cli
---

```gherkin
Feature: staged binding action

  Scenario: a clean binding
    Then it is previewed with the previous value and the diff, written only without --dry-run, and only the changed line differs

  Scenario: refused bindings
    Then a value outside the range, outside the narrowing, at the wrong stage or outside the enumeration is refused with the validator's code, once each, and nothing is written

  Scenario: spelling, unknown targets and the MCP tool
    Then an existing key spelling is kept, an unknown target is refused with a suggestion, and the MCP tool previews by default and refuses a bad value even when asked to write
```
