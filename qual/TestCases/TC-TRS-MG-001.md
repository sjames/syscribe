---
id: TC-TRS-MG-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the refines link on UseCaseDef: write/parse, refinedBy index, E316 bad target, W307 missing (draft-suppressed), magicgrid profile promotion."
verifies:
  - REQ-TRS-MG-001
---

```gherkin
Feature: UseCaseDef refines link to requirements
  Scenario: a refines link parses and back-links the requirement
    Given a UseCaseDef with refines naming a Requirement by id and another by qualified name
    When the model is validated
    Then both targets resolve and each requirement reports the use case under refinedBy

  Scenario: an unresolved refines target raises E316
    Given a UseCaseDef whose refines names an operand that resolves to nothing
    When the model is validated
    Then E316 is raised against the use case

  Scenario: a refines target that is not a requirement raises E316
    Given a UseCaseDef whose refines names a PartDef
    When the model is validated
    Then E316 is raised naming the resolved type

  Scenario: a non-draft use case with no refines warns W307
    Given a UseCaseDef at status approved with no refines link
    When the model is validated
    Then W307 is raised and the default exit code is unaffected

  Scenario: a draft use case with no refines is suppressed
    Given a UseCaseDef at status draft with no refines link
    When the model is validated
    Then no W307 finding is produced

  Scenario: the magicgrid profile promotes W307 to a gate failure
    Given a non-draft UseCaseDef with no refines link and a [profiles.magicgrid] gate
    When validate --profile magicgrid is run
    Then the command exits non-zero
```
