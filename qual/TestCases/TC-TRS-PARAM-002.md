---
id: TC-TRS-PARAM-002
type: TestCase
testLevel: L3
status: draft
name: "Verify parameterConstraints evaluation: E221/W025, compound appliesWhen, dotted refs."
verifies:
  - REQ-TRS-PARAM-002
---

```gherkin
Feature: parameterConstraints numeric evaluation
  Scenario: a violated constraint under a holding appliesWhen is an error
    Given an AMP configuration binding maxCpus = 1 and a constraint maxCpus >= 2 appliesWhen Amp
    When feature-check runs
    Then E221 names the violating configuration
  Scenario: severity:warning violations are W025; appliesWhen that does not hold is not evaluated
  Scenario: compound appliesWhen is boolean-parsed (no spurious W014); clean model is silent
```
