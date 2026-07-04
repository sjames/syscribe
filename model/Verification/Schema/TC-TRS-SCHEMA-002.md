---
type: TestCase
id: TC-TRS-SCHEMA-002
name: "reqClass is parsed and does not raise the unrecognized-field warning"
status: draft
testLevel: L2
tags:
  - schema
  - requirements
verifies:
  - REQ-TRS-SCHEMA-002
---

Verifies that `reqClass` on a `Requirement` is recognized by the parser (bound to the
element model, not the unknown-field catch-all) and therefore does not trigger `W047`.

```gherkin
Feature: reqClass is a recognized field

  Scenario: A Requirement with reqClass validates without W047
    Given a Requirement whose frontmatter declares "reqClass: stakeholder"
    When the model is validated
    Then no W047 warning is reported for that Requirement
    And the parsed reqClass value equals "stakeholder"

  Scenario: reqClass survives a frontmatter-only round-trip
    Given a Requirement with "reqClass: system"
    When its frontmatter is re-serialized
    Then the emitted frontmatter still contains "reqClass: system"
```
