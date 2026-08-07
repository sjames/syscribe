---
id: TC-TRS-HPLE-004
type: TestCase
testLevel: L3
status: draft
name: "Verify an unresolved required parameter anywhere in a consolidated subtree is reported as an opt-in warning, gateable via --deny, never a hard error."
verifies:
  - REQ-TRS-HPLE-004
---

```gherkin
Feature: Open-parameter completeness across a consolidated subtree
  Scenario: a still-open required parameter reachable through subConfigurations is a warning
    Given a Configuration whose subConfigurations subtree leaves a selected, required, no-default parameter unbound
    When the model is validated
    Then a warning is raised naming that parameter, and the exit code stays zero

  Scenario: the same warning is gateable via --deny
    Given the same model as above
    When the model is validated with that warning code denied
    Then the exit code is non-zero

  Scenario: a fully-closed subtree raises no such warning
    Given a Configuration whose subConfigurations subtree has every open parameter bound by some tier on the path
    When the model is validated
    Then no open-parameter warning is raised
```
