---
id: TC-TRS-BT-032
type: TestCase
testLevel: L3
status: draft
name: "Verify a dependent relying on a retained element's absence is reported"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-032.sh
verifies:
  - REQ-TRS-BT-032
---

```gherkin
Feature: Dependents of retained elements
  Scenario: relying on absence
    Given a test that verifies the absence of a retained element
    When validate --config is run
    Then a finding is raised; a test that does not rely on absence is silent
```
