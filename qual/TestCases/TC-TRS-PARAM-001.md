---
id: TC-TRS-PARAM-001
type: TestCase
testLevel: L3
status: draft
name: "Verify FeatureDef parameter binding rules E203–E206, E222, and W017."
verifies:
  - REQ-TRS-PARAM-001
---

Verify that Configuration `parameterBindings` are validated against `FeatureDef` parameter declarations, and that fully valid bindings produce no findings.

```gherkin
Feature: FeatureDef parameter binding validation

  Scenario: binding a parameter of an unselected feature produces E203
    Given a Configuration that deselects a feature but binds its parameter
    When the tool validates the model
    Then an E203 finding is emitted

  Scenario: binding a fixed parameter produces E204
    Given a Configuration that binds a parameter declared isFixed/value
    When the tool validates the model
    Then an E204 finding is emitted

  Scenario: an out-of-range value produces E205
    Given a Configuration that binds a value outside the parameter range
    When the tool validates the model
    Then an E205 finding is emitted

  Scenario: a value not in enumValues produces E206
    Given a Configuration that binds a value not in the parameter enumValues
    When the tool validates the model
    Then an E206 finding is emitted

  Scenario: an unresolved binding path produces E222
    Given a Configuration that binds an undeclared parameter
    When the tool validates the model
    Then an E222 finding is emitted

  Scenario: a required, unbound parameter on a selected feature produces W017
    Given a Configuration that selects a feature but omits its required parameter
    When the tool validates the model
    Then a W017 finding is emitted

  Scenario: fully valid bindings produce no parameter findings
    Given a Configuration with selected feature, in-range, in-enum, all required bound
    When the tool validates the model
    Then no E203, E204, E205, E206, E222, or W017 finding is emitted
```
