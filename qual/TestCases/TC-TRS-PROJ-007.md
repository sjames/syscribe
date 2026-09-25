---
id: TC-TRS-PROJ-007
type: TestCase
testLevel: L3
status: draft
name: "Verify validate gating flags, --profile and --file in the configuration lens, and validate's usage-error exit code."
verifies:
  - REQ-TRS-OUT-006
  - REQ-TRS-PROJ-005
  - REQ-TRS-PROJ-001
---

Issue #126: the gating options, `--profile` and `--file` apply under `--config` and per variant under
`--all-configs`, with errors dominating gates (`1` > `2` > `0`); usage errors on `validate` exit `1`,
never the gate code `2`.

```gherkin
Feature: Gating in the configuration lens

  Scenario: --all-configs honours the gating flags per variant
    Given a feature model with two warning-only variants
    When validate --all-configs runs with no gating flag
    Then it exits 0
    And with --warnings-as-errors, --deny W015 or --max-warnings 0 it exits 2
    And with --max-warnings 6 (between the two variants' warning counts) it exits 2
    And with --deny W999 (absent everywhere) it exits 0
    And the summary marks the tripped variant as a gate failure

  Scenario: --all-configs honours a scoped --profile per variant
    Given a profile promoting W300 only on elements tagged wdt, and a wdt-tagged requirement active only in one variant
    When validate --all-configs --profile wdt runs
    Then it exits 2 and only that variant is marked as a gate failure

  Scenario: errors dominate gates across variants
    Given a configuration family where one variant has an error
    When validate --all-configs --warnings-as-errors runs
    Then it exits 1

  Scenario: --config honours the gating flags and --profile
    When validate --config <Wdt variant> --warnings-as-errors runs
    Then it exits 2
    And validate --config <Wdt variant> --profile wdt exits 2
    And validate --config <NoWdt variant> --profile wdt exits 0

  Scenario: --config honours --file
    When validate --config <Wdt variant> --file Reqs/Wdt.md runs
    Then only findings for Reqs/Wdt.md are reported

  Scenario: usage errors exit 1
    When validate --config names an unresolvable configuration
    Then it exits 1 with a stderr message and empty stdout
    And validate --profile <undefined> exits 1 with empty stdout
```
