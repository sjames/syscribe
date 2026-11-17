---
id: TC-TRS-CALCBUILD-001
type: TestCase
testLevel: L3
status: active
name: "Verify: build-config exports the result of a CalculationDef through its buildVar."
verifies:
  - REQ-TRS-CALCBUILD-001
sourceFile: repo:crates/syscribe/tests/derived_calcs.rs
testFunctions:
  - build_config_exports_calculation_results_in_every_format
  - build_config_unit_integral_failure_and_conflict
---

```gherkin
Feature: build-config exports the result of a CalculationDef through its buildVar (TC-TRS-CALCBUILD-001)

  Scenario: The behaviour of REQ-TRS-CALCBUILD-001 holds
    Given a model built to exercise it
    When the command runs
    Then the result is as the requirement states
```
