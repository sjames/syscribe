---
id: TC-TRS-SAFE-012
type: TestCase
testLevel: L3
status: draft
name: "Verify ASIL D / SIL 4 decomposition pair completeness: E865 when siblings share a satisfies target, W860 for a single-child decomposition, clean when distinct; decompositionKind surfaces in the safety-case report and the Requirement template; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SAFE-012
---

Verify the decomposition structural checks and the `decompositionKind` reporting.

```gherkin
Feature: ASIL D / SIL 4 decomposition pair completeness (E865, W860)

  Scenario: E865 — decomposition siblings share a satisfies target
    Given an ASIL D parent with two ASIL B children that both satisfy the same element
    When the tool validates the model
    Then an E865 finding is emitted

  Scenario: no E865 — siblings satisfy distinct elements
    Given an ASIL D parent with two ASIL B children that satisfy distinct elements
    When the tool validates the model
    Then no E865 finding is emitted

  Scenario: W860 — single-child decomposition
    Given an ASIL D parent with only one lower-level child
    When the tool validates the model
    Then a W860 finding is emitted

  Scenario: W860 draft-suppressed
    Given a single-child decomposition whose parent is draft
    When the tool validates the model
    Then no W860 finding is emitted

  Scenario: --deny W860 promotes to a gate failure
    Given the single-child decomposition
    When the tool validates the model with --deny W860
    Then the tool exits non-zero

  Scenario: decompositionKind appears in the safety-case report
    Given a requirement that sets decompositionKind
    When the safety-case report is generated
    Then the report shows the decomposition kind for that requirement

  Scenario: template Requirement includes a decompositionKind hint
    When the Requirement template is printed
    Then it contains a commented decompositionKind line
```
