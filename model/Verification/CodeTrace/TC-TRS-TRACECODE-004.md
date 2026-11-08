---
type: TestCase
id: TC-TRS-TRACECODE-004
name: "trace-code checks a [repos] peer against its own elements"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/trace_code.rs
verifies:
  - REQ-TRS-TRACECODE-004
tags:
  - multi-repo
---

```gherkin
Feature: composed repositories

  Scenario: a peer with its own [traceCode]
    Given a peer repository with its own model, code and [traceCode] table
    And a crate in it named only by an element of the composing model
    When trace-code runs in the composing model
    Then the peer is reported separately and that crate is untraced there
    And the command exits 2
```
