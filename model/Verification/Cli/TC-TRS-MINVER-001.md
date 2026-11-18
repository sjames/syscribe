---
id: TC-TRS-MINVER-001
type: TestCase
testLevel: L3
status: active
name: "Verify an older syscribe refuses a model that declares a newer minVersion, the minimum is reported, and W329 flags a newer feature."
verifies:
  - REQ-TRS-MINVER-001
sourceFile: repo:crates/syscribe/tests/min_version.rs
testFunctions:
  - an_older_syscribe_refuses_the_model_naming_both_versions
  - a_model_whose_minimum_is_met_loads_and_commands_without_a_model_still_run
  - a_malformed_minimum_is_an_error_not_a_pass
  - policy_and_capabilities_report_the_declared_minimum
  - validate_warns_when_the_model_uses_a_feature_newer_than_its_minimum
---

```gherkin
Feature: Minimum syscribe version (TC-TRS-MINVER-001)

  Scenario: An older syscribe refuses the model
    Given a model declaring a minVersion newer than the running syscribe
    When any model command, mcp or lsp runs
    Then it exits 1 naming both versions

  Scenario: The minimum is reported and a newer feature is flagged
    Given a model declaring minVersion 0.60.0 that uses a derived parameter
    When validate and policy run
    Then validate raises W329 and policy reports the minimum
```
