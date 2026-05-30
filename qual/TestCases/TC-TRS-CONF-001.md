---
id: TC-TRS-CONF-001
type: TestCase
testLevel: L3
status: draft
title: "Verify that E200, E201, and E209 are emitted for Configuration and appliesWhen violations."
verifies:
  - REQ-TRS-CONF-001
---

Verify that Configuration element validation rules and the appliesWhen cross-reference rule are enforced.

```gherkin
Feature: Configuration element and appliesWhen validation

  Scenario: bad Configuration id produces E200
    Given a Configuration element with id: CFG-001 (not matching CONF-* pattern)
    When the tool validates the model
    Then an E200 finding is emitted

  Scenario: Configuration missing featureModel produces E201
    Given a Configuration element with id, title, and status but no featureModel
    When the tool validates the model
    Then an E201 finding is emitted

  Scenario: appliesWhen referencing a non-FeatureDef produces E209
    Given an element with appliesWhen: pointing to a Part element
    When the tool validates the model
    Then an E209 finding is emitted
```
