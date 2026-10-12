---
id: TC-TRS-FM-008
type: TestCase
testLevel: L3
status: draft
name: "Verify dotted tree paths resolve in featureTree requires/excludes, and unresolved feature references name the accepted form and closest match."
verifies:
  - REQ-TRS-FM-008
---

```gherkin
Feature: feature reference forms
  Scenario: dotted tree path in requires/excludes
    Given a featureTree: entry with requires: [Protection.Afci] and the qname and FEAT- forms elsewhere
    When the tool runs `feature-check`
    Then no E212 is reported
  Scenario: unresolved references name the form and closest match
    Given an appliesWhen, a Configuration features key, a parameterBindings key and a placeholder each written in the dotted form
    And a requires entry with a typo
    When the tool runs `validate` and `feature-check`
    Then E209, E239, E222, E241 and E212 each say which form the field accepts
    And each suggests the feature it most likely meant
```
