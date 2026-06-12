---
id: TC-TRS-ELEM-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that implicit base library supertypes are applied when no supertype: is given."
verifies:
  - REQ-TRS-ELEM-003
---

Verify that implicit base library supertypes are applied when no supertype: is given.

```gherkin
Feature: Implicit supertype application

  Scenario: PartDef with no supertype: is treated as specializing Parts::Part
    Given a PartDef element with no supertype: field
    When the tool constructs the element graph
    Then the element's resolved supertype is Parts::Part (or compatible implicit default)
    And no E004 error is emitted for missing supertype:

  Scenario: Explicit supertype: overrides the implicit default
    Given a PartDef element with supertype: MyBase
    And MyBase exists in the model
    When the tool constructs the element graph
    Then the element's resolved supertype is MyBase, not Parts::Part
```
