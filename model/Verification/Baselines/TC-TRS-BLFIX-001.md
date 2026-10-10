---
type: TestCase
id: TC-TRS-BLFIX-001
name: "baseline detail diff, relative manifests, superseded verify and --current drift listing"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline_detail.rs
verifies:
  - REQ-TRS-BLFIX-001
tags:
  - baseline
---

```gherkin
Feature: baseline fixes

  Scenario: relative manifest paths
    Given a baseline created in a git repository whose model root is a subdirectory
    Then manifest elements[].file is relative to the git root

  Scenario: detail diff
    Given two baselines around a requirement edit
    Then baseline diff --detail prints the removed and added lines

  Scenario: legacy absolute manifest
    Given a manifest with absolute file paths
    Then baseline diff --detail still prints the lines

  Scenario: superseded
    Given a superseded baseline whose scope has drifted
    Then verify --all reports it skipped and exits 0

  Scenario: current drift
    Given a modified working tree
    Then baseline diff BL --current and verify BL --detail list the changed element
```
