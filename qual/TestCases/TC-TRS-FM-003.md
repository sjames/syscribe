---
id: TC-TRS-FM-003
type: TestCase
testLevel: L3
status: draft
name: "Verify feature-check parameter rules E207, E202, E213, W014."
verifies:
  - REQ-TRS-FM-003
---

Verify derivedFrom cycle detection, bindTo propagation range, and parameterConstraints path/appliesWhen checks under `feature-check`.

```gherkin
Feature: Feature model parameter integrity

  Scenario: parameter violations emit E207, E202, E213, W014
    Given a FeatureDef with a circular derivedFrom, a bindTo value out of the
      component range, a parameterConstraints expression with an unresolved path,
      and a constraint appliesWhen feature selected in no configuration
    When the tool runs feature-check
    Then findings E207, E202, E213, and W014 are emitted

  Scenario: a clean feature model emits none of them
    Given an acyclic derivedFrom, an in-range bindTo value, resolvable constraint
      paths, and constraint features present in some configuration
    When the tool runs feature-check
    Then none of E207, E202, E213, W014 are emitted
```
