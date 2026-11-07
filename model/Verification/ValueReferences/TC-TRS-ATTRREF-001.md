---
type: TestCase
id: TC-TRS-ATTRREF-001
name: "Value references render with their unit in show, export-html and scenarios"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/attr_refs.rs
verifies:
  - REQ-TRS-ATTRREF-001
tags:
  - references
---

Scenario text below writes references inside inline code so that this TestCase's own body
does not resolve them.

```gherkin
Feature: Value references in prose and scenarios

  Scenario: every attribute form renders
    Given a PartDef with a feature value with unit B, a nested feature, an Attribute child, a tagged value with a default and a ram budget
    And a TestCase body and gherkin scenario using `{{PolicydBounds.snapshotFixedBytes}}` and the other forms
    When show renders the TestCase
    Then the body and the scenario read "47244 B", "64 KiB", "128" and "12 KiB"
    And a non-gherkin fenced block and an inline code span keep the raw reference
    And the file on disk still holds the raw reference

  Scenario: export-html
    When export-html writes the TestCase page
    Then the page holds "47244 B"

  Scenario: feature-parameter placeholders are not taken over
    Given a reference by qualified name `{{Design::Limits.maxFrame}}`
    Then validate reports neither E240 nor E241 for it
```
