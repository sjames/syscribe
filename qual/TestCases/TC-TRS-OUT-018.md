---
id: TC-TRS-OUT-018
type: TestCase
testLevel: L3
status: draft
name: "Verify behavioral-coverage: source-overlap (path 1) and allocation (path 4) coverage, active-only by default, --include-planned, --uncovered-only, json schema, correct percentage, and the demo model >50%."
verifies:
  - REQ-TRS-OUT-018
---

Verify the `behavioral-coverage` report against a fixture exercising two coverage paths plus
the flags, and confirm the demo model's out-of-the-box coverage.

```gherkin
Feature: Behavioral coverage (§20)

  Scenario: source-overlap coverage (path 1)
    Given an ActionDef implementedBy a dir and an active TestCase whose sourceFile is under it
    When `behavioral-coverage` is run
    Then that ActionDef is covered

  Scenario: allocation coverage (path 4)
    Given an active TestCase verifying a requirement satisfied by a part allocated to an ActionDef
    When `behavioral-coverage` is run
    Then that ActionDef is covered

  Scenario: uncovered element
    Given an ActionDef with no covering active test
    When `behavioral-coverage` is run
    Then it is reported uncovered

  Scenario: only active tests count by default
    Given an ActionDef covered only by a draft TestCase
    When `behavioral-coverage` is run
    Then it is uncovered

  Scenario: --include-planned surfaces planned coverage
    When `behavioral-coverage --include-planned` is run
    Then the draft TestCase appears in the planned column

  Scenario: --uncovered-only filters but keeps the true percentage
    When `behavioral-coverage --uncovered-only` is run
    Then only uncovered elements are listed and the coverage total is unchanged

  Scenario: json output matches the schema
    When `behavioral-coverage --format json` is run
    Then it has coverage_pct and per-element coveredBy

  Scenario: demo model achieves >50% out of the box
    Given the shipped demo model
    When `behavioral-coverage` is run
    Then coverage is above 50%
```
