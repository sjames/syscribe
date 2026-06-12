---
id: TC-TRS-XREF-005
type: TestCase
testLevel: L3
status: draft
name: "Verify that verifies: and derivedFrom: references are resolved by stable id:."
verifies:
  - REQ-TRS-XREF-005
---

Verify that verifies: and derivedFrom: references are resolved by stable id:.

```gherkin
Feature: ID-based cross-reference resolution

  Scenario: verifies: resolved by id: field regardless of file path
    Given a Requirement with id: REQ-TRS-TEST-001 located at an arbitrary path
    And a TestCase with verifies: [REQ-TRS-TEST-001]
    When the tool is invoked
    Then the verifies: reference resolves without an E102 error

  Scenario: derivedFrom: resolved by id: field
    Given a parent Requirement with id: REQ-PARENT-001
    And a child Requirement with derivedFrom: [REQ-PARENT-001]
    And the parent and child are in different subdirectories
    When the tool is invoked
    Then derivedFrom: resolves without an E103 error
```
