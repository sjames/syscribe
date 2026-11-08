---
type: TestCase
id: TC-TRS-AMEND-002
name: "W324: edits since acceptance without an amendment"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/amendments.rs
verifies:
  - REQ-TRS-AMEND-002
tags:
  - adr
  - amendments
---

```gherkin
Feature: Edits to accepted elements

  Scenario: uncommitted edit
    Given require_amendments = true and an accepted ADR
    When its body is edited in the working tree
    Then W324 names it as an uncommitted edit, and an amendment dated today clears it

  Scenario: committed edit
    When the edit is committed with an amendment dated before it
    Then W324 names the changing commit, and an amendment dated on or after it clears it

  Scenario: not flagged
    Then edits before acceptance, an amendment-only change and a status set in the working tree raise nothing

  Scenario: confirmed requirement
    Then an approved requirement is guarded, and moving on to implemented keeps the guard

  Scenario: opt-in and git
    Then nothing is raised without the setting or outside a repository, and a malformed value is W632
```
