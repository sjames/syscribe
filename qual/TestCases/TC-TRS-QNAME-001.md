---
id: TC-TRS-QNAME-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that qualified names are derived correctly from directory path and filename stem."
verifies:
  - REQ-TRS-QNAME-001
---

Verify that qualified names are derived correctly from directory path and filename stem.

```gherkin
Feature: Qualified name derivation from path

  Scenario: Single-level element has single-segment qualified name
    Given a file model/Engine.md with type: PartDef
    When the tool is invoked with model/ as root
    Then the element has qualified name Engine

  Scenario: Three-level nested element has three-segment qualified name
    Given a file model/VehicleSystem/Powertrain/Engine.md with type: PartDef
    When the tool is invoked with model/ as root
    Then the element has qualified name VehicleSystem::Powertrain::Engine
```
