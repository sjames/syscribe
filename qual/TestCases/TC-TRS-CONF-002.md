---
id: TC-TRS-CONF-002
type: TestCase
testLevel: L3
status: draft
name: "Verify Configuration selection parsing: template uses features:, W016 on empty selections, show displays selections."
verifies:
  - REQ-TRS-CONF-002
---

Verify template/parser agreement on `Configuration.features:`, the `W016` no-silent-ignore warning, and that `show` displays parsed selections.

```gherkin
Feature: Configuration selection parsing

  Scenario: template emits the canonical features: map
    Given the Configuration template
    When the tool prints it
    Then it contains a features: map
    And it does not contain a selections: key

  Scenario: legacy selections: under a feature model warns (W016)
    Given a FeatureDef and a Configuration using a selections: block
    When the tool validates the model
    Then a W016 finding is emitted

  Scenario: a features:-map configuration does not warn
    Given a FeatureDef and a Configuration using a features: map
    When the tool validates the model
    Then no W016 finding is emitted

  Scenario: empty selections without a feature model is silent
    Given a Configuration using selections: and no FeatureDef in the model
    When the tool validates the model
    Then no W016 finding is emitted

  Scenario: show displays parsed feature selections
    Given a Configuration with a features: map
    When the tool shows that configuration
    Then the output lists the selected feature
```
