---
type: TestCase
id: TC-TRS-ATTRREF-003
name: "A literal copy of a linked attribute value is W095 when copied_values is on"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/attr_refs.rs
verifies:
  - REQ-TRS-ATTRREF-003
tags:
  - references
---

```gherkin
Feature: W095

  Scenario: opt-in copied value
    Given a TestCase that verifies PolicydBounds directly and states "47 244 bytes" in its scenario
    When validate runs without [references] copied_values
    Then no W095 is reported
    When [references] copied_values = true
    Then W095 names PolicydBounds.snapshotFixedBytes and suggests the reference

  Scenario: noise filter
    Given attributes with the values 8 and 2026 and prose stating 8 retries in 2026 and the id TC-47244
    Then no W095 is reported for them

  Scenario: malformed config
    Given [references] copied_values = "yes"
    Then W094 on .syscribe.toml says it must be true or false

  Scenario: lint-docs
    Given an external document referencing PolicydBounds with a value reference and stating 47244
    Then lint-docs reports W095 and exits 0 (advisory)
```
