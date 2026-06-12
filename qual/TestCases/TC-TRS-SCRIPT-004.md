---
id: TC-TRS-SCRIPT-004
type: TestCase
testLevel: L3
status: draft
name: "Verify the two registration shapes (register_command and register_check), a pure library file, and the duplicate-name load error."
verifies:
  - REQ-TRS-SCRIPT-004
---

Verify that a single file can register both a command and a check, that both are surfaced,
that a `.rhai` file registering nothing is a pure library (not runnable), and that two
scripts registering the same command name produce a deterministic duplicate-name error.

```gherkin
Feature: registration shapes

  Scenario: a file registers both a command and a check
    Given a script calling register_command and register_check
    When "scripts list" is run
    Then both the command and the check are listed with their kinds

  Scenario: a pure library is not runnable
    Given a .rhai file that registers nothing
    When "scripts list" is run
    Then it does not appear as a runnable command or check

  Scenario: duplicate name is a load error
    Given two scripts registering the same command name
    When any scripts subcommand is run
    Then a duplicate-name error is reported and the exit code is non-zero
```
