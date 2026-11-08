---
type: TestCase
id: TC-TRS-ITEMREF-003
name: "Open and answered question citations are checked"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/item_refs.rs
verifies:
  - REQ-TRS-ITEMREF-003
tags:
  - references
---

```gherkin
Feature: Question citations

  Scenario: committed element cites an open question
    Given an approved and a draft requirement citing open question PI-IR-001#Q1
    Then only the approved one raises W093

  Scenario: answered question with to-confirm wording
    Given a sentence 'to be confirmed (PI-IR-001#Q2)' where Q2 is answered
    Then W094 is raised
    And a configured [references] open_markers list replaces the defaults
    And a malformed open_markers value is W094 on .syscribe.toml
```
