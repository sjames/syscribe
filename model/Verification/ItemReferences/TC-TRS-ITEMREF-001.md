---
type: TestCase
id: TC-TRS-ITEMREF-001
name: "Item anchors are parsed, displayed without the token and checked"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/item_refs.rs
verifies:
  - REQ-TRS-ITEMREF-001
tags:
  - references
---

```gherkin
Feature: Requirement item anchors

  Scenario: display
    Given a requirement whose first item begins with {#grant-keys}
    When show and export-html render it
    Then the item keeps its number, the token is gone, and export-html has id="item-grant-keys"

  Scenario: anchor problems
    Given a requirement defining {#a} twice and a malformed {#Bad}
    Then validate reports W092 'defined twice' and 'malformed anchor'
```
