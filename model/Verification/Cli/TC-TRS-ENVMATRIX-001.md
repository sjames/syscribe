---
type: TestCase
id: TC-TRS-ENVMATRIX-001
name: "The environment matrix shows verdict, currency and gaps per rig"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/env_matrix.rs
verifies:
  - REQ-TRS-ENVMATRIX-001
tags:
  - cli
---

```gherkin
Feature: Requirement x TestEnvironment matrix

  Scenario: a simulator run and no target-rig run
    Then the requirement shows pass in one column and a gap in the other, and rolls up as a gap

  Scenario: evidence on both rigs
    Then each cell shows its own verdict, build commit and date, a failure on one rig fails the roll-up and --gaps-only lists it

  Scenario: stale evidence
    Then a result for a build older than the last change to the requirement is marked stale, and so is one whose requirement changed since ingest

  Scenario: unusable rigs and unknown builds
    Then a retired or expired rig's results do not count, and a build commit the repository does not know is noted, not judged stale

  Scenario: surfaces
    Then --json, --status, --tag, --config and the MCP coverage_matrix by environment agree
```
