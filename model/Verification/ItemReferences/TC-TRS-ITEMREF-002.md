---
type: TestCase
id: TC-TRS-ITEMREF-002
name: "Dangling item and question references are W092"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/item_refs.rs
verifies:
  - REQ-TRS-ITEMREF-002
tags:
  - references
---

```gherkin
Feature: Item and question references

  Scenario: valid and dangling references
    Given a requirement citing REQ-IR-001#grant-keys, #3, item 2 and PI-IR-001#Q2
    And citing #nope, #9, Item 7 and PI-IR-001#Q9
    Then exactly the four dangling references are W092
    And references inside fenced code and to unresolved elements are not reported
```
