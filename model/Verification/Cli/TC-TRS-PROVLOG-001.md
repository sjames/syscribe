---
type: TestCase
id: TC-TRS-PROVLOG-001
name: "Attributed changes are committed to git and read back by log"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/provenance_log.rs
verifies:
  - REQ-TRS-PROVLOG-001
tags:
  - cli
---

```gherkin
Feature: Provenance in git

  Scenario: a committed change
    Then the actor is the commit author and the trailers carry the command, actor and reason

  Scenario: only the changed files
    Then another actor's staged or unstaged work is never committed

  Scenario: optional and opt-in
    Then no actor means the configured identity, and nothing is committed without --commit or the config switch, or on a dry run

  Scenario: log
    Then it shows who did what and why for an element, filters by actor and date, lists hand edits as unattributed and follows a rename
```
