---
type: TestCase
id: TC-TRS-REQMATRIX-001
name: "the matrix model lists requirement rows, category columns and direct-edge kinds"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQMATRIX-001
tags:
  - server
---

```gherkin
Feature: explorer matrix lens

  Scenario: matrix model
    Then rows are Requirement nodes, columns the chosen category, cells the direct edge kinds in either direction, and unlinked rows are flagged

  Scenario: page controls
    Then the page has the matrix toggle, the column selector and the matrix container
```
