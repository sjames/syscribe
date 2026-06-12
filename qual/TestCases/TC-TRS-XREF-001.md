---
id: TC-TRS-XREF-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that absolute qualified names are resolved correctly from the model root."
verifies:
  - REQ-TRS-XREF-001
---

Verify that absolute qualified names are resolved correctly from the model root.

```gherkin
Feature: Absolute cross-reference resolution

  Scenario: Absolute supertype reference resolves correctly
    Given an element A::B::Child with supertype: A::B::Parent
    And an element A::B::Parent exists in the model
    When the tool is invoked
    Then no unresolved-reference error is emitted for Child's supertype

  Scenario: Absolute reference to non-existent element produces an error
    Given an element with supertype: A::B::Nonexistent
    And no element with qualified name A::B::Nonexistent in the model
    When the tool is invoked
    Then an unresolved-reference error is emitted for that supertype reference
```
