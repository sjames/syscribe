---
type: TestCase
id: TC-TRS-IMPACTFILES-001
name: "impact maps changed files to elements and flags the unreviewed"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/impact_files.rs
verifies:
  - REQ-TRS-IMPACTFILES-001
tags:
  - cli
---

```gherkin
Feature: impact from changed files

  Scenario: a source file bound by implementedBy
    Then its unit, the requirement it satisfies and the test that verifies it are listed

  Scenario: files with no element
    Then they are listed separately as not mapped

  Scenario: every binding kind
    Then own file, implementedBy (file and directory), sourceFile and evidence path each map, and a deleted file still maps

  Scenario: since a revision
    Then changed and not-changed elements are marked and --fail-on-unreviewed exits 2 until the test moves with the code

  Scenario: reach and errors
    Then depth and direction apply, and bad input exits 1

  Scenario: MCP
    Then the impact tool accepts files and since
```
