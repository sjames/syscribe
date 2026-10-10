---
type: TestCase
id: TC-TRS-TESTENV-001
name: "TestEnvironment validates and runsOn is checked against capabilities and calibration"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/test_environment.rs
verifies:
  - REQ-TRS-TESTENV-001
tags:
  - verification
---

```gherkin
Feature: test environments

  Scenario: a valid environment and runsOn
    Then validate reports none of E893, E894, W891, W892

  Scenario: defects
    Then a bad enum, a non-environment runsOn target, a missing capability and an expired or retired rig are reported
```
