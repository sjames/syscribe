---
type: TestCase
id: TC-TRS-REQURL-001
name: "view state round-trips through the URL query and invalid values fall back"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQURL-001
tags:
  - server
---

```gherkin
Feature: explorer URL view state

  Scenario: round trip
    Then every valid combination of layout, view, trace and cols survives build then parse, and defaults are omitted

  Scenario: invalid values
    Then unknown values fall back to the defaults without error
```
