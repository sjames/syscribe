---
type: TestCase
id: TC-TRS-RUNHIST-001
name: "named runs are retained and diffed"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_history.rs
verifies:
  - REQ-TRS-RUNHIST-001
  - REQ-TRS-AGENTPOL-005
tags:
  - cli
---

```gherkin
Feature: run history

  Scenario: no run id
    Then ingest without --run writes no history

  Scenario: two runs
    Then runs lists both and diff reports regressions, fixed and still failing

  Scenario: replace and errors
    Then re-ingesting a run id replaces it, an unknown id exits 1 and --fail-on-regression gates

  Scenario: records in their own file
    Then a records ingest writes a slim record, without steps, artefacts or operator but with measurements, hashes and finish time, to the run's file while the index holds only the count, and a later ingest of the same run supersedes its keys without touching other runs' files

  Scenario: a function-level run
    Then it writes no run file and keeps the index's shape

  Scenario: the cap
    Then with keep_run_records set the oldest run's file is deleted and ingest says so, its verdicts stay so results diff still compares it, results runs shows records dropped, a re-ingested run counts as newest, 0 keeps none and a malformed value warns and uses the default

  Scenario: a moved runs_dir
    Then the run files follow the setting, an absolute or parent-relative value is resolved by name, and a value that would let a run replace the sidecar is refused with the default applied

  Scenario: file names and unreadable files
    Then a run id with a slash and a space becomes a safe file name while the index keeps the id, and an unreadable run file is refused and left alone

  Scenario: the history goes through the write layer
    Then a dry run writes neither the history nor the sidecar, and a protected history or run file is refused with the rule and file named before anything is written
```
