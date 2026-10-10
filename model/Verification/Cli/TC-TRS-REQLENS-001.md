---
type: TestCase
id: TC-TRS-REQLENS-001
name: "table rows, CSV quoting with formula guard and export file names are pure and correct"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQLENS-001
tags:
  - server
---

```gherkin
Feature: explorer table lens and export

  Scenario: table rows
    Then elements carry hop distance from the root and relations list from, to and kind

  Scenario: CSV
    Then cells with commas, quotes and newlines are quoted, and formula-leading cells are neutralised

  Scenario: file names and page controls
    Then the root is reduced to a safe name and the page has the view toggle and export buttons
```
