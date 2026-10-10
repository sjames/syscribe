---
type: TestCase
id: TC-TRS-PHOLD-001
name: "placeholders are substituted per configuration and validated"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/placeholders.rs
verifies:
  - REQ-TRS-PHOLD-001
tags:
  - variability
---

```gherkin
Feature: parameter placeholders

  Scenario: substitution
    Given configurations binding a parameter to different values
    Then each projection shows its own value, with the unit for |unit, and the base model keeps the placeholder

  Scenario: fallbacks
    Then a fixed value or default is used when a configuration does not bind the parameter

  Scenario: gate
    Then a placeholder without a feature model or configuration is E240

  Scenario: bad reference
    Then an unknown feature or parameter is E241

  Scenario: gating escape
    Then an ungated element referencing a feature that some configuration does not select is E242

  Scenario: unbound
    Then draft elements get W245 and approved elements E243

  Scenario: runtime
    Then a runtime parameter reference is W246
```
