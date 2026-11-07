---
type: TestCase
id: TC-TRS-ATTRREF-005
name: "refs Elem.attr lists uses, literal copies and stale copies"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/attr_refs.rs
verifies:
  - REQ-TRS-ATTRREF-005
tags:
  - references
  - cli
---

```gherkin
Feature: refs Elem.attr

  Scenario: text output
    Given a TestCase using the reference, a Requirement stating 47244 and linked to PolicydBounds, and a stale 46 852
    When refs PolicydBounds.snapshotFixedBytes runs
    Then it prints the value 47244 B and sections for uses, literal copies and stale copies naming each element

  Scenario: json
    When refs PolicydBounds.snapshotFixedBytes --json runs
    Then the object has the attribute, value, unit and the uses, copies and stale arrays
```
