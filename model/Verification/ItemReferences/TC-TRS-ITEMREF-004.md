---
type: TestCase
id: TC-TRS-ITEMREF-004
name: "refs, impact, check-ref and lint-docs accept item references"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/item_refs.rs
verifies:
  - REQ-TRS-ITEMREF-004
tags:
  - references
---

```gherkin
Feature: Item reference queries

  Scenario: refs
    Then refs REQ-IR-001#grant-keys lists citations by anchor and by number of that item only
    And refs REQ-IR-001 shows an 'Item and question citations' section
    And refs PI-IR-001#Q1 lists the citing element

  Scenario: impact and check-ref
    Then impact REQ-IR-001#rotate lists only the element citing that item
    And check-ref resolves anchors, 'item N' and questions, and exits 1 on a missing anchor

  Scenario: lint-docs
    Given an external note citing REQ-IR-001#gone
    Then lint-docs reports W092 and exits 1
```
