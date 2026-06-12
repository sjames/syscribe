---
id: TC-TRS-SCRIPT-002
type: TestCase
testLevel: L3
status: draft
title: "Verify extension scripts run sandboxed and resource-limited: runaway aborted, eval and filesystem escape refused, parse errors named without crashing siblings, output deterministic."
verifies:
  - REQ-TRS-SCRIPT-002
---

Verify that a runaway script is aborted by the operation limit (bounded time, non-zero),
that `eval` and filesystem/module-escape attempts are refused, that a syntactically
invalid script is reported with its name while sibling scripts still load and run, and
that running the same command twice yields identical output.

```gherkin
Feature: sandboxed, resource-limited, deterministic execution

  Scenario: a runaway loop is aborted by the operation limit
    Given a command whose function never terminates
    When it is run via "scripts run"
    Then the tool aborts it with an error and a non-zero exit (no hang)

  Scenario: eval is disabled
    Given a command that calls eval
    When it is run
    Then the run fails with a non-zero exit

  Scenario: a module import escaping the scripts dir fails
    Given a command importing a path outside the scripts directory
    When it is run
    Then the run fails rather than reading outside the directory

  Scenario: a parse error is reported with the script name and does not break siblings
    Given one syntactically invalid script and one valid command
    When "scripts list" is run
    Then the invalid script is reported by name
    And the valid command is still listed and runnable

  Scenario: deterministic output
    Given a healthy command
    When it is run twice over the same model
    Then both runs produce identical output
```
