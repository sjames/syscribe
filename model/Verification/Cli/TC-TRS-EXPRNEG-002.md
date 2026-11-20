---
id: TC-TRS-EXPRNEG-002
type: TestCase
testLevel: L3
status: active
name: "Verify a negative Celsius value is accepted as a recorded measurement value and limit."
verifies:
  - REQ-TRS-EXPRNEG-001
sourceFile: repo:crates/syscribe/tests/results_records_edge.rs
testFunctions:
  - measurement_values_parse_in_the_usual_notations
---

```gherkin
Feature: TC-TRS-EXPRNEG-002

  Scenario: records
    Given the model or expression the tests build
    When the command runs
    Then the result is as the requirement states
```
