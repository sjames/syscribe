---
id: TC-TRS-XREF-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that an unresolved cross-reference produces an error but does not abort processing."
verifies:
  - REQ-TRS-XREF-003
---

Verify that an unresolved cross-reference produces an error but does not abort processing.

```gherkin
Feature: Unresolved reference is non-fatal

  Scenario: Dangling supertype reference produces an error without crashing
    Given a model with one element containing a supertype: reference to a non-existent element
    And a second element that is valid
    When the tool is invoked
    Then a reference error is emitted for the dangling reference
    And the valid element is still counted in the element total
    And the tool exits normally (not with a crash or panic)
```
