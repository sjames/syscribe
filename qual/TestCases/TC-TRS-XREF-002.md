---
id: TC-TRS-XREF-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that relative references are resolved outward from the current package."
verifies:
  - REQ-TRS-XREF-002
---

Verify that relative references are resolved outward from the current package.

```gherkin
Feature: Relative cross-reference resolution

  Scenario: Sibling reference resolves within the same package
    Given element Pkg::Child with supertype: Parent
    And element Pkg::Parent in the same package
    When the tool is invoked
    Then Child's supertype resolves to Pkg::Parent without error

  Scenario: ./prefix restricts resolution to siblings only
    Given element Pkg::Sub::Child with supertype: ./Parent
    And element Pkg::Sub::Parent in the same package
    When the tool is invoked
    Then the reference resolves to Pkg::Sub::Parent
```
