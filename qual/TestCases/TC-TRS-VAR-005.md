---
id: TC-TRS-VAR-005
type: TestCase
testLevel: L3
status: draft
name: "Verify per-Configuration uncovered-requirement rule W015 and its suppression/gating."
verifies:
  - REQ-TRS-VAR-005
---

Verify that `W015` is emitted for a requirement active in a configuration with no covering in-config `TestCase`, is suppressed for draft elements and draft-test coverage, is gateable, and is dormant with no feature model.

```gherkin
Feature: Per-Configuration coverage validation

  Scenario: active uncovered requirement yields W015 naming requirement and configuration
    Given REQ-V5-WDT-002 is active in CONF-MPS2-WDT-001 with no covering TestCase
    When the tool validates the model
    Then a W015 finding naming REQ-V5-WDT-002 and CONF-MPS2-WDT-001 is emitted

  Scenario: covered requirement yields no W015
    Given REQ-V5-WDT-001 is covered by a TestCase that runs in CONF-MPS2-WDT-001
    Then no W015 finding names REQ-V5-WDT-001

  Scenario: draft requirement is not flagged
    Given REQ-V5-WDT-003 is draft and active but uncovered
    Then no W015 finding names REQ-V5-WDT-003

  Scenario: a draft TestCase does not count as coverage
    Given REQ-V5-WDT-004 is covered only by a draft TestCase
    Then a W015 finding names REQ-V5-WDT-004

  Scenario: W015 is gateable
    Given the model emits at least one W015
    When the tool validates with --deny W015
    Then it exits with code 2

  Scenario: dormant model emits no W015
    Given a model with zero FeatureDef elements
    Then no W015 finding is emitted
```
