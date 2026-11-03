---
type: TestCase
id: TC-TRS-HINTS-001
name: "typos and hand edits are answered with the right command"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/help_discovery_and_hints.rs
verifies:
  - REQ-TRS-HINTS-001
tags:
  - cli
---

```gherkin
Feature: Did-you-mean and workflow hints

  Scenario: a set typo
    Then set PI-X stauts=done answers "did you mean status" and status=in-progress suggests in_progress

  Scenario: unknown command or option
    Then the nearest commands or the nearest option are named

  Scenario: explain
    Then explain E106 prints the catalogue text and an unknown code exits 1

  Scenario: validate --hints
    Then I910, I911 and I912 name claim/release, the duplicate entry and the set command, and are absent without the flag
```
