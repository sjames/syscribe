---
id: TC-TRS-FM-001
type: TestCase
testLevel: L3
status: draft
title: "Verify the feature-check command: discoverable, exit codes, dormancy, --json."
verifies:
  - REQ-TRS-FM-001
---

Verify the explicit `feature-check` command exists, is separate from `validate`, uses the 0/1 exit-code contract, is dormant with no feature model, and supports `--json`.

```gherkin
Feature: feature-check command

  Scenario: feature-check is discoverable in help
    When the tool prints --help
    Then feature-check is listed

  Scenario: a clean feature model exits 0
    Given a feature model with no violations
    When the tool runs feature-check
    Then it exits 0

  Scenario: a feature model with a violation exits 1
    Given a feature model with a structural violation
    When the tool runs feature-check
    Then it exits 1

  Scenario: no feature model is dormant
    Given a model with no FeatureDef
    When the tool runs feature-check
    Then it prints a "no feature model present" notice and exits 0

  Scenario: --json emits structured findings
    Given a feature model with violations
    When the tool runs feature-check --json
    Then the output is a JSON array of findings
```
