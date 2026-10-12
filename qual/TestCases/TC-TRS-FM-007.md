---
id: TC-TRS-FM-007
type: TestCase
testLevel: L3
status: draft
name: "Verify a mandatory group with optional children is not reported as W012; an optional feature still is."
verifies:
  - REQ-TRS-FM-007
---

```gherkin
Feature: W012 respects mandatory membership
  Scenario: mandatory group with optional children
    Given a FeatureDef with mandatory: true and groupKind: optional
    And every Configuration selects it
    When the tool runs `feature-check`
    Then no W012 is reported
  Scenario: optional feature selected everywhere
    Given a FeatureDef with groupKind: optional and no mandatory field
    And every Configuration selects it
    When the tool runs `feature-check`
    Then W012 is reported and suggests mandatory: true
  Scenario: grid-tie acceptance model
    Given the bundled model_gti/ product line
    When the tool runs `feature-check`
    Then no W012 is reported and the feature model has no findings
```
