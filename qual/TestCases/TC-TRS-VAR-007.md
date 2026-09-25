---
id: TC-TRS-VAR-007
type: TestCase
testLevel: L3
status: draft
name: "Verify Configuration inheritance through derivedFrom: effective selection, overrides, consumers and structural checks."
verifies:
  - REQ-TRS-VAR-007
  - REQ-TRS-HPLE-001
---

Issue #137: spec §9.8 documents Configuration inheritance through `derivedFrom:`, but a
Configuration with `derivedFrom:` raised `E105`. This case exercises the inherited selection
through every consumer and each structural check.

```gherkin
Feature: Configuration inheritance through derivedFrom

  Scenario: an inheriting Configuration validates cleanly
    Given an approved base selecting Wdt (binding its required timeout) and deselecting Log
    And children that add Log, deselect Wdt by feature id, declare nothing, or override the timeout two levels down
    When validate runs
    Then it exits 0 with no E105, E017, W016, W017 or E203

  Scenario: projection uses the effective selection
    When list Requirement runs with --config naming each child
    Then the child adding Log shows both the Wdt and the Log requirement
    And the bare child shows exactly what the base shows
    And the child deselecting Wdt shows neither

  Scenario: matrix, configure, show and all-configs use the effective selection
    When matrix --json runs
    Then the Wdt requirement is applicable (not N/A) in the bare child's column
    When configure runs on the bare child
    Then no feature is left free
    When show runs on a child
    Then its inherited entries are marked (inherited)
    When validate --all-configs runs
    Then it exits 0

  Scenario: a child's own binding overrides the inherited one
    Given a child binding the timeout out of range over an in-range base binding
    When validate runs
    Then E205 is raised on the child and not on the base

  Scenario: a consolidated peer Configuration inherits the binding that closes its parameter
    Given a consolidating tier whose subConfigurations names an inheriting peer Configuration
    When validate runs
    Then there is no E518 and no W513

  Scenario: structural problems are reported with dedicated codes
    Given configurations with a dangling base, a FeatureDef base, a two-member cycle, two bases, and a draft base
    When validate runs
    Then E234, E235, E236 (on both members), E237 and E215 are raised
    And E105 and E017 are not raised
```
