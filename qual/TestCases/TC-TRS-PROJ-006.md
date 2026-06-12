---
id: TC-TRS-PROJ-006
type: TestCase
testLevel: L3
status: draft
name: "Verify the --config projection lens on metrics, cyber-risk, co-analysis, verification-depth and safety-case."
verifies:
  - REQ-TRS-PROJ-006
---

```gherkin
Feature: Configuration lens on read-only analysis commands

  Scenario: each analysis command accepts a valid --config and rejects a bad one
    Given a variant model with a feature model and a configuration
    When metrics/cyber-risk/co-analysis/verification-depth/safety-case is run with --config <valid>
    Then the command exits 0
    When the same command is run with --config <unresolvable>
    Then the command exits non-zero

  Scenario: verification-depth is computed over the projected active subset
    Given a SIL-3 requirement gated by appliesWhen to an unselected feature
    When verification-depth is run whole-model
    Then the gated requirement appears in the report
    When verification-depth is run with --config <the config that excludes the feature>
    Then the gated requirement is absent (projected out)
```
