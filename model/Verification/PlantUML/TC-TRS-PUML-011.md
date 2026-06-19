---
type: TestCase
id: TC-TRS-PUML-011
name: "E403 is emitted when pumlMode has an unrecognized value"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-032
---

Verifies that the validator emits error `E403` when a `Diagram` element's `pumlMode` field
contains a value that is not the recognized string `companion`.

```gherkin
Feature: E403 validation for unrecognized pumlMode value

  Scenario: pumlMode set to an unrecognized value triggers E403
    Given a Diagram element with pumlMode: generate (not a recognized value)
    When syscribe -m <root> validate is run
    Then the output contains E403
    And the E403 message mentions pumlMode
    And the E403 message mentions the unrecognized value generate
```
