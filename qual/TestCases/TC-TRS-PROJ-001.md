---
id: TC-TRS-PROJ-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the --config projection lens: stored + ad-hoc selection, dormancy, unresolved error."
verifies:
  - REQ-TRS-PROJ-001
---

```gherkin
Feature: Configuration projection lens
  Scenario: lens filters to active elements (stored configuration)
    Given REQ-CORE (always) and REQ-WDT (appliesWhen Wdt)
    When listing requirements with --config the Wdt configuration
    Then REQ-CORE and REQ-WDT are listed
    And with --config the no-Wdt configuration REQ-WDT is excluded
  Scenario: ad-hoc feature selection
    When listing requirements with --config "Features::Wdt"
    Then REQ-WDT is listed
  Scenario: --config is a usage error with no feature model
    Given a model with no FeatureDef and no Configuration named X
    When any command that accepts --config is run with --config X
    Then it exits non-zero, prints nothing on stdout, and stderr says the model declares no feature model
    And the same command without --config still succeeds
  Scenario: --config naming a stored Configuration is accepted with no feature model
    Given a model with no FeatureDef but a stored Configuration C (a parametric variant)
    When validate and list are run with --config C
    Then they succeed with the same output as without --config
  Scenario: unresolved configuration errors
    Given a feature model is present
    When --config names an unknown configuration
    Then the tool exits non-zero
```
