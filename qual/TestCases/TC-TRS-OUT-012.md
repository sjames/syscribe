---
id: TC-TRS-OUT-012
type: TestCase
testLevel: L3
status: draft
name: "Verify named, SIL/ASIL-scopable validation severity profiles and their exit codes."
verifies:
  - REQ-TRS-OUT-012
---

Verify that `validate --profile <name>` loads a `[profiles.<name>]` table from `<model_root>/.syscribe.toml`, promotes the listed warning codes to gate failures (exit `2`), honours an optional `sil`/`status`/`tag` scope so only matching findings are promoted, and exits `1` for an undefined profile.

```gherkin
Feature: Named, SIL/ASIL-scopable validation severity profiles

  Scenario: Warnings without a profile exit zero
    Given a model that emits W300 warnings and no errors
    When validate is invoked with no profile and no gating flags
    Then the exit code is 0

  Scenario: An unscoped profile promotes every listed code
    Given a model that emits two W300 warnings (one on a SIL-4 element, one on a non-SIL element)
    And a profile "all300" with promote = ["W300"] and no scope
    When validate is invoked with --profile all300
    Then the exit code is 2

  Scenario: A SIL-scoped profile promotes only matching findings
    Given the same model
    And a profile "safety" with promote = ["W300"] and sil = "4"
    When validate is invoked with --profile safety
    Then the exit code is 2

  Scenario: A profile whose scope matches nothing promotes nothing
    Given the same model
    And a profile "none" with promote = ["W300"] and tag = "does-not-exist"
    When validate is invoked with --profile none
    Then the exit code is 0

  Scenario: An undefined profile name is an error
    Given the same model
    When validate is invoked with --profile nonexistent
    Then the exit code is 1
```
