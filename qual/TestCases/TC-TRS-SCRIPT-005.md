---
id: TC-TRS-SCRIPT-005
type: TestCase
testLevel: L3
status: draft
title: "Verify scripts list and scripts run: enumeration with kind/description/source, running a command (text and --json), unknown-name failure, and check-not-runnable-as-command."
verifies:
  - REQ-TRS-SCRIPT-005
---

Verify that `scripts list` enumerates each registration's name, kind, description and
source file (plain and `--json`), that `scripts run <command>` prints the command's
returned string, that an unknown command name exits non-zero, and that running a check
name via `scripts run` reports that it is a check (not a command).

```gherkin
Feature: scripts list and scripts run

  Scenario: list shows a command and a check
    Given a model with a registered command and check
    When "scripts list" is run
    Then each is shown with its kind, description, and source file
    And "scripts list --json" carries the same fields

  Scenario: run a command
    Given a registered command
    When "scripts run <command>" is run
    Then the command's returned string is printed

  Scenario: unknown command
    Given no command of that name
    When "scripts run nope" is run
    Then it exits non-zero with a message

  Scenario: a check is not runnable via scripts run
    Given a registered check
    When "scripts run <check>" is run
    Then it reports that the name is a check (run under scripts validate) and exits non-zero

  Scenario: no scripts directory
    Given a model with no scripts directory
    When "scripts list" is run
    Then it reports that none are defined and exits 0
```
