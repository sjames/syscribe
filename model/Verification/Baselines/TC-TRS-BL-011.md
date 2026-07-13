---
type: TestCase
id: TC-TRS-BL-011
name: "Configuration-projected scope seals and drift-checks the variant"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - scope
  - variability
verifies:
  - REQ-TRS-BL-011
---

Verifies that `frozenScope.config` projects the model to a variant before sealing, that the
seal covers only the active elements, and that a change to the variant (or its Configuration)
is detected as drift.

```gherkin
Feature: Configuration-projected baseline scope

  Scenario: A config-scoped seal covers only the active variant
    Given a model with a feature model and an element gated by appliesWhen off in CONF-A
    When `baseline create --tag REL --frozen-scope "config=CONF-A"` is run
    Then the sealed elementCount excludes the inactive element
    And the manifest lists only variant-active elements

  Scenario: Two configs seal different content
    Given elements gated to different features
    When one baseline seals config=CONF-A and another seals config=CONF-B
    Then their aggregate hashes differ

  Scenario: Changing an active in-variant element drifts the config-scoped baseline
    Given a released config-scoped baseline
    When an element active in that variant is changed
    Then validation reports E520 for that baseline

  Scenario: A change confined to an out-of-variant element does not drift
    Given a released baseline scoped to CONF-A
    When an element inactive in CONF-A is changed
    Then no drift is reported for that baseline

  Scenario: An unresolvable config is refused
    Given `--frozen-scope "config=CONF-NONEXISTENT"`
    When create is run
    Then it refuses and writes nothing
```
