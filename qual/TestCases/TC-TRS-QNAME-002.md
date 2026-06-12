---
id: TC-TRS-QNAME-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that the name: field in _index.md overrides the directory name in qualified names."
verifies:
  - REQ-TRS-QNAME-002
---

Verify that the name: field in _index.md overrides the directory name in qualified names.

```gherkin
Feature: Package name override via _index.md

  Scenario: name: in _index.md replaces the directory name
    Given a directory VehicleSystem/ with _index.md containing name: VS
    And a file VehicleSystem/Engine.md with type: PartDef
    When the tool is invoked
    Then Engine.md has qualified name VS::Engine

  Scenario: Absent name: in _index.md uses directory name
    Given a directory VehicleSystem/ with _index.md that omits the name: field
    And a file VehicleSystem/Engine.md with type: PartDef
    When the tool is invoked
    Then Engine.md has qualified name VehicleSystem::Engine
```
