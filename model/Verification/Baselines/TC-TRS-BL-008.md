---
type: TestCase
id: TC-TRS-BL-008
name: "baseline verify proves content and git consistency"
status: draft
testLevel: L2
tags:
  - baseline
  - cli
verifies:
  - REQ-TRS-BL-008
---

Verifies `baseline verify`: the content proof (recomputed aggregate == seal == manifest),
the git tag↔commit consistency check, and read-only behaviour.

```gherkin
Feature: baseline verify

  Scenario: verify passes for an intact baseline
    Given a baseline whose scope content is unchanged and whose tag points at its commit
    When `baseline verify BL-2026-07` is run
    Then it reports success and exits zero

  Scenario: verify fails on content drift
    Given a baseline whose in-scope content changed since sealing
    When `baseline verify BL-2026-07` is run
    Then it reports the content-proof failure and exits non-zero

  Scenario: verify fails on tag/commit mismatch
    Given a baseline whose gitTag resolves to a different commit than gitCommit
    Then `baseline verify` reports the git-consistency failure and exits non-zero

  Scenario: A missing tag is a warning, not a failure
    Given a baseline whose gitTag does not yet exist
    Then `baseline verify` warns about the missing tag but does not fail on that alone

  Scenario: verify --all gates every baseline
    Given several baselines, one drifted
    When `baseline verify --all` is run
    Then it exits non-zero and identifies the drifted baseline

  Scenario: verify is read-only
    When verify is run
    Then no model file is modified
```
