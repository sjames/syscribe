---
id: TC-TRS-FMA-004
type: TestCase
testLevel: L3
status: draft
name: "Verify full-semantics configuration validity (E225) without duplicating E219/E220."
verifies:
  - REQ-TRS-FMA-004
---

```gherkin
Feature: Configuration validity under full semantics
  Scenario: structural violations are E225
    Given configurations violating alternative count, a mandatory feature, and child-parent
    When feature-check --deep runs
    Then each is reported as E225 and a valid configuration is not
  Scenario: requires violations stay E219
    Given a configuration violating a requires
    Then it is reported E219 and not duplicated as E225
```
