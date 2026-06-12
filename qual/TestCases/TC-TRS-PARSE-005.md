---
id: TC-TRS-PARSE-005
type: TestCase
testLevel: L3
status: draft
name: "Verify that _index.md is treated as the package declaration for its directory."
verifies:
  - REQ-TRS-PARSE-005
---

Verify that _index.md is treated as the package declaration for its directory.

```gherkin
Feature: _index.md as package declaration

  Scenario: _index.md metadata applies to the package
    Given a directory Pkg/ containing _index.md with type: Package
    And a sibling file Pkg/Foo.md with type: PartDef
    When the tool is invoked against the model root
    Then the element at Pkg/Foo.md has qualified name Pkg::Foo
    And _index.md itself does not appear as a child element named _index

  Scenario: name: in _index.md overrides the directory name
    Given a directory VehicleSystem/ with _index.md containing name: VS
    And a file VehicleSystem/Engine.md with type: PartDef
    When the tool is invoked
    Then Engine.md has qualified name VS::Engine
```
