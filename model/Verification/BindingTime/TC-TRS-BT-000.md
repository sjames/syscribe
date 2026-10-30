---
id: TC-TRS-BT-000
type: TestCase
testLevel: L4
status: draft
name: "Verify a staged product line end to end: compile, load and runtime Configurations, residual, exported parameter file and its check"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-000.sh
verifies:
  - REQ-TRS-BT-000
---

```gherkin
Feature: Staged configuration from declaration to the running system
  Scenario: a compile, a load and a runtime Configuration chain validates and reports residual variability
    Given a feature model with compile-, load- and runtime-window parameters and a three-Configuration derivedFrom chain at ascending stages
    When the model is validated and residual is run on each Configuration
    Then validate is clean and each residual lists only the later-stage items

  Scenario: the runtime Configuration exports a parameter file that checks clean
    Given the runtime-stage Configuration
    When params export then params check are run on the output
    Then the file validates against the generated schema and params check exits zero

  Scenario: a late edit to the model makes the exported file stale
    Given an exported file and a later change to a parameter's range
    When params check is run on the old file
    Then a staleness warning is reported
```
