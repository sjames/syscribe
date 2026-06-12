---
id: TC-TRS-ID-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that duplicate id: values across elements produce E101."
verifies:
  - REQ-TRS-ID-004
---

Verify that duplicate id: values across elements produce E101.

```gherkin
Feature: Duplicate id detection

  Scenario: Two Requirement elements with the same id produce E101
    Given two Requirement files both carrying id: REQ-DUP-001
    When the tool is invoked
    Then an E101 finding is emitted for the duplicate id

  Scenario: Two elements of different types with the same id produce E101
    Given a Requirement with id: REQ-DUP-001 and a TestCase with id: REQ-DUP-001
    When the tool is invoked
    Then an E101 finding is emitted

  Scenario: Unique ids across all elements produce no E101
    Given a model where every element has a distinct id: value
    When the tool is invoked
    Then no E101 finding is emitted
```
