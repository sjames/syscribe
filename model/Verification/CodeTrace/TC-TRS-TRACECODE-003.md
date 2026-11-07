---
type: TestCase
id: TC-TRS-TRACECODE-003
name: "validate raises W326 and W327 only with [traceCode] validate = true"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/trace_code.rs
verifies:
  - REQ-TRS-TRACECODE-003
tags:
  - validation
---

```gherkin
Feature: trace-code in validate

  Scenario: off by default
    Given a [traceCode] table without validate = true
    Then validate raises no W326, W327 or W328

  Scenario: switched on
    Given validate = true and a baseline with one live and one traced entry
    Then validate raises W326 for the new unit and W327 for the traced entry, exits 0
    And --deny W326 and --deny W327 each exit 2

  Scenario: malformed entry
    Then validate raises W328
```
