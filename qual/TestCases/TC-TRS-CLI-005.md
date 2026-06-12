---
id: TC-TRS-CLI-005
type: TestCase
testLevel: L3
status: draft
name: "Verify detailed per-command help: help <cmd>, <cmd> --help, the index, and unknown handling."
verifies:
  - REQ-TRS-CLI-005
---

Verify every command has a detailed man page reachable as `help <cmd>` and `<cmd> --help`, that the index lists commands, and that an unknown command exits non-zero — all without a model directory.

```gherkin
Feature: detailed per-command help

  Scenario: every command has a man page via help <cmd>
    Given the set of dispatchable commands
    When the tool runs `help <cmd>` for each
    Then each prints a man page containing SYNOPSIS and exits 0

  Scenario: <cmd> --help prints the same page without a model
    When the tool runs `<cmd> --help` and `<cmd> -h`
    Then each prints the command's man page and exits 0

  Scenario: help with no argument prints an index
    When the tool runs `help`
    Then it lists multiple commands with one-line summaries

  Scenario: help on an unknown command exits non-zero
    When the tool runs `help bogus-command`
    Then it exits non-zero
```
