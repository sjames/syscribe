---
id: TC-TRS-VAR-003
type: TestCase
testLevel: L3
status: draft
title: "Verify boolean expressions (and/or/not/parens) in appliesWhen parse and evaluate."
verifies:
  - REQ-TRS-VAR-003
---

Verify that `appliesWhen:` accepts boolean expressions over `FeatureDef` qualified names, evaluates them correctly against a configuration's selections, keeps bare-QName behaviour, and reports an unresolved operand as `E209`.

```gherkin
Feature: Boolean appliesWhen expressions

  Scenario: unresolved operand inside an expression is E209
    Given a requirement with appliesWhen "Features::A and Features::Nope"
    And Features::A resolves but Features::Nope does not
    When the tool validates the model
    Then an E209 finding is emitted

  Scenario: AND evaluates against selections
    Given a requirement with appliesWhen "Features::A and Features::B"
    Then it is active only in the configuration selecting both A and B

  Scenario: OR evaluates against selections
    Given a requirement with appliesWhen "Features::A or Features::B"
    Then it is active in every configuration selecting A or B, N/A only when neither

  Scenario: NOT evaluates against selections
    Given a requirement with appliesWhen "not Features::A"
    Then it is active exactly in the configurations that deselect A

  Scenario: bare QName remains back-compatible
    Given a requirement with appliesWhen Features::A
    Then it is active exactly in the configurations that select A
```
