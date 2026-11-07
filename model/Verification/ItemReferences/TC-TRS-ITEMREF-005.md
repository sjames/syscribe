---
type: TestCase
id: TC-TRS-ITEMREF-005
name: "Item-granular suspect links"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/item_refs.rs
verifies:
  - REQ-TRS-ITEMREF-005
tags:
  - references
---

```gherkin
Feature: Item-level baselines

  Scenario: only the cited item makes the link suspect
    Given REQ-IR-010 cites REQ-IR-001#grant-keys and the link is accepted
    When another item is edited and a new item is inserted above
    Then no W090 is raised
    When the cited item's text changes
    Then W090 names REQ-IR-001#grant-keys and suspect list shows it

  Scenario: by-number references are refused
    Then suspect accept REQ-IR-011 REQ-IR-001#3 exits 1 and says to add an anchor
```
