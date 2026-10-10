---
type: TestCase
id: TC-TRS-FAILNOTE-001
name: "trace and safety-case print the retained failure message"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/failure_notes.rs
verifies:
  - REQ-TRS-FAILNOTE-001
tags:
  - cli
---

```gherkin
Feature: failure notes

  Scenario: trace
    Then a failing verifier's function, message and time are listed under the table

  Scenario: safety-case
    Then the text and the json carry the same failure details and a passing model prints none
```
