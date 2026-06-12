---
id: TC-TRS-PROJ-005
type: TestCase
testLevel: L3
status: draft
name: "Verify family checks: all-configs gate, dead elements (W021), aggregate coverage (W022), diff."
verifies:
  - REQ-TRS-PROJ-005
---

```gherkin
Feature: Family-level checks
  Scenario: validate --all-configs gates on any variant error
    Given two configurations, one with a structural escape
    When running validate --all-configs
    Then both configurations are listed and the tool exits non-zero
  Scenario: dead element
    Given a Part appliesWhen a feature unsatisfiable under the feature model
    When running feature-check --deep
    Then W021 is reported for that element
  Scenario: aggregate coverage gap
    Given a requirement active in some configuration but covered in none
    Then W022 is reported
  Scenario: variant diff
    When running diff --config A --config B
    Then it reports the requirements active in one but not the other
```
