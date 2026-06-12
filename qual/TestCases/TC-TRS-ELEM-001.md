---
id: TC-TRS-ELEM-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that all element types defined in §2 are recognised and processed without E005."
verifies:
  - REQ-TRS-ELEM-001
---

Verify that all element types defined in §2 are recognised and processed without E005.

```gherkin
Feature: Element type inventory

  Scenario Outline: Each defined element type is recognised
    Given a .md file with type: <element_type> and the minimum required fields
    When the tool is invoked
    Then no E005 finding is emitted for that file

    Examples:
      | element_type       |
      | PartDef            |
      | Part               |
      | ItemDef            |
      | Item               |
      | PortDef            |
      | Port               |
      | InterfaceDef       |
      | Interface          |
      | ConnectionDef      |
      | Connection         |
      | ActionDef          |
      | Action             |
      | AttributeDef       |
      | RequirementDef     |
      | Requirement        |
      | TestCase           |
      | ADR                |
      | Allocation         |
      | ViewDef            |
      | View               |
      | Package            |
```
