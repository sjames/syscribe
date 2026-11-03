---
type: TestCase
id: TC-TRS-DRYRUN-001
name: "Every writing command leaves the tree untouched under --dry-run"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/dry_run.rs
verifies:
  - REQ-TRS-DRYRUN-001
tags:
  - cli
---

```gherkin
Feature: Global dry run

  Scenario: each writer
    Then a real run changes the tree and the same run with --dry-run, before or after the command, does not

  Scenario: truthful messages
    Then a dry run says Would write, never Wrote
```
