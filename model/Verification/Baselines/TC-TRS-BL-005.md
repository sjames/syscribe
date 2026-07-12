---
type: TestCase
id: TC-TRS-BL-005
name: "Drift is status-graded and released baselines are frozen"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - validation
verifies:
  - REQ-TRS-BL-005
---

Verifies drift detection with status-graded severity (`W520`/`E520`), the seal-vs-manifest
tamper check (`E521`), the unresolved-supersedes error (`E522`), and the supersession
immutability rule. Each severity level is a distinct scenario for mechanical testability.

```gherkin
Feature: Baseline drift and freeze

  Scenario: A fresh baseline is clean
    Given a just-sealed baseline whose scope is unchanged
    When the model is validated
    Then no drift finding is emitted

  Scenario: Draft drift is silent
    Given a draft baseline and a changed in-scope element
    Then validation emits no drift finding

  Scenario: Approved drift is a warning
    Given an approved baseline and a changed in-scope element
    Then validation emits W520

  Scenario: Released drift is an error
    Given a released baseline and a changed in-scope element
    Then validation emits E520

  Scenario: A superseded baseline is not drift-checked
    Given a baseline with status superseded whose content has moved on
    Then validation emits no drift finding for it

  Scenario: A tampered seal is an error
    Given a Baseline whose seal.aggregateHash disagrees with its manifest
    Then validation emits E521

  Scenario: An unresolved supersedes reference is an error
    Given a Baseline whose supersedes names no existing baseline
    Then validation emits E522
```
