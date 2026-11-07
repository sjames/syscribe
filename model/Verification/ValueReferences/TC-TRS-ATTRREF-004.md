---
type: TestCase
id: TC-TRS-ATTRREF-004
name: "A literal equal to a previous value of a linked attribute is W096"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/attr_refs.rs
verifies:
  - REQ-TRS-ATTRREF-004
tags:
  - references
---

```gherkin
Feature: W096 stale copies

  Scenario: a budget changed after the TestCase copied it
    Given a git repository where PolicydBounds.snapshotFixedBytes was committed as 46852 and later as 47244
    And an active TestCase linked to PolicydBounds whose scenario states "46 852 bytes"
    When validate runs
    Then W096 names the previous value 46852 and the current value 47244

  Scenario: no git, or a draft element
    Given the same files outside a git repository, or the TestCase in draft
    Then no W096 is reported
```
