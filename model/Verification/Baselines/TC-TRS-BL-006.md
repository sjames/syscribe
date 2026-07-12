---
type: TestCase
id: TC-TRS-BL-006
name: "baseline diff reports added, removed, and changed by stable id"
status: draft
testLevel: L2
tags:
  - baseline
  - cli
verifies:
  - REQ-TRS-BL-006
---

Verifies `baseline diff`: element-level add/remove/change keyed by stable id, offline from
manifests, grouped by type, with an optional git-backed `--detail` view.

```gherkin
Feature: baseline diff

  Scenario: Element-level diff from manifests
    Given two baselines BL-2026-06 and BL-2026-07
    When `baseline diff BL-2026-06 BL-2026-07` is run
    Then elements only in BL-2026-07 are reported added
    And elements only in BL-2026-06 are reported removed
    And elements in both with a differing hash are reported changed
    And the results are grouped by element type

  Scenario: Diff keys on stable id across a file move
    Given an element that moved files between BL-2026-06 and BL-2026-07 but kept its id
    Then it is reported changed (or unchanged), not as removed+added

  Scenario: Hash-level diff needs no source control
    Given both manifests are present
    Then `baseline diff` produces its result without any git access

  Scenario: --detail shows field-level change via git
    Given both baselines' commits are retrievable
    When `baseline diff BL-2026-06 BL-2026-07 --detail` is run
    Then the field/body-level change of each changed element is shown
    And an unretrievable element degrades gracefully to the hash-level result
```
