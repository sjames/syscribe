---
id: TC-TRS-BT-019
type: TestCase
testLevel: L3
status: active
name: "Verify feature-check --deep detects an unsatisfiable staged chain"
verifies:
  - REQ-TRS-BT-019
---

```gherkin
Feature: Staged-chain satisfiability
  Scenario: dead end
    Given a compile Configuration that leaves no valid completion of a later feature
    When feature-check --deep is run
    Then an error is raised naming that Configuration

  Scenario: satisfiable chain
    Given a staged chain with valid completions
    When feature-check --deep is run
    Then it is clean
```
