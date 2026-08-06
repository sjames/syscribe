---
id: TC-TRS-PLANITEM-004
type: TestCase
testLevel: L3
status: draft
name: "Verify a PlanningItem gated by appliesWhen: projects in/out across Configurations, and the feature model stays sound."
verifies:
  - REQ-TRS-PLANITEM-004
---

```gherkin
Feature: PlanningItem appliesWhen:
  Scenario: a gated PlanningItem is active under the selecting Configuration
    Given a PlanningItem with appliesWhen: <FEAT-id> and a Configuration selecting that feature true
    When why-active is run against that Configuration
    Then the verdict is active

  Scenario: a gated PlanningItem is inactive under the non-selecting Configuration
    Given the same PlanningItem and a Configuration selecting that feature false
    When why-active is run against that Configuration
    Then the verdict is inactive

  Scenario: feature-check --deep reports the feature model as sound
    Given the same model
    When feature-check --deep is run
    Then it reports the feature model is not void and both Configurations are valid
```
