---
type: TestCase
id: TC-TRS-BL-100
name: "End-to-end baseline lifecycle: create, verify, drift, supersede, diff"
status: draft
testLevel: L3
tags:
  - baseline
  - release-management
  - integration
verifies:
  - REQ-TRS-BL-000
---

Integration test exercising the full release-baseline lifecycle the feature promises: a
scope is sealed and verifies clean, a later change makes a released baseline drift (error), a
superseding baseline restores a clean state, and the two baselines diff meaningfully.

```gherkin
Feature: Baseline lifecycle

  Scenario: Seal, then verify clean
    Given a model on a clean git tree
    When `baseline create --tag REL-2026-06 --approver "J. Roe"` seals the scope as BL-2026-06
    Then `baseline verify BL-2026-06` passes
    And validation reports no drift

  Scenario: A change makes a released baseline drift
    Given BL-2026-06 is marked released
    When an in-scope element is changed
    Then validation reports E520 against the released baseline

  Scenario: A superseding baseline restores a clean state
    When `baseline create --tag REL-2026-07` seals the new state as BL-2026-07 with supersedes BL-2026-06
    And BL-2026-06 is marked superseded
    Then validation reports no drift (BL-2026-06 is historical, BL-2026-07 is current)

  Scenario: The two baselines diff meaningfully
    When `baseline diff BL-2026-06 BL-2026-07` is run
    Then the changed element is reported under its type, keyed by stable id
```
