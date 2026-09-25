---
id: TC-TRS-ADR-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the ADR deciders: field is accepted without W047 and shown by show, and stays W047 on other types."
verifies:
  - REQ-TRS-ADR-001
---

Verify the §8.17.1 `ADR` `deciders:` field (GH #159).

```gherkin
Feature: ADR deciders

  Scenario: deciders on an ADR is a recognized field
    Given ADR-DEC-001 with deciders Stakeholders::SystemsEngineer and the free-text name "Jane Doe"
    When the tool validates the model
    Then no W047 finding names ADR-DEC-001.md
    And no finding names the free-text decider "Jane Doe"

  Scenario: show displays the deciders and the date
    When the user runs show ADR-DEC-001
    Then the output has a deciders row listing Stakeholders::SystemsEngineer and Jane Doe
    And a date row with 2026-05-20

  Scenario: deciders on a non-ADR element is still unrecognized
    Given the PartDef Stakeholders::SystemsEngineer carrying deciders
    When the tool validates the model
    Then a W047 finding names SystemsEngineer.md and the field 'deciders'
```
