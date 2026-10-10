---
type: TestCase
id: TC-TRS-REQEXPL-001
name: "the Requirements Explorer page is served, seeded safely and laid out in layers"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQEXPL-001
tags:
  - server
---

```gherkin
Feature: requirements explorer page

  Scenario: the page and the navigation
    Then /requirements serves the explorer and every page links to it

  Scenario: seeding
    Then focus, depth and config are escaped data attributes and a bad depth falls back

  Scenario: layout
    Then the client layout function puts the root in column zero and each hop in the next column without overlaps
```
