---
type: TestCase
id: TC-TRS-REQSEARCH-001
name: "the explorer search API ranks matches and the client filter hides non-matching nodes"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQSEARCH-001
tags:
  - server
---

```gherkin
Feature: explorer search and filters

  Scenario: search ranking and limits
    Then an exact id outranks an id prefix, which outranks a name match, matching is case-insensitive, and limit truncates

  Scenario: search errors
    Then an empty query, a bad limit and an unknown configuration are 400

  Scenario: client filter
    Then filtering by type, verification and ASIL hides non-matching nodes and their edges but never the root
```
