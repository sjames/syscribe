---
id: TC-TRS-SCRIPT-006
type: TestCase
testLevel: L3
status: draft
name: "Verify scripts validate: namespaced <check>/<code> findings, the 0/1/2 exit contract with gate flags, and independence from the built-in validate."
verifies:
  - REQ-TRS-SCRIPT-006
---

Verify that `scripts validate` runs the registered checks, prints findings namespaced as
`<check>/<code>` with the source script, exits 1 on an error-severity finding and 0 on a
clean run, honours `--deny`/`--max-warnings`/`--warnings-as-errors` (exit 2 on a tripped
gate), and that the built-in `validate` is byte-for-byte unaffected by the presence of
check scripts (and does not run them).

```gherkin
Feature: scripts validate

  Scenario: findings are namespaced and exit 1 on an error
    Given a check emitting an error-severity finding
    When "scripts validate" is run
    Then findings render as <check>/<code> with the source script
    And the exit code is 1

  Scenario: clean run exits 0
    Given a model whose checks emit no error-severity findings
    When "scripts validate" is run
    Then the exit code is 0

  Scenario: gate flags trip exit 2
    Given a check emitting a warning-severity finding
    When "scripts validate --deny <check>/<code>" is run on an otherwise clean model
    Then the exit code is 2

  Scenario: built-in validate is unaffected
    Given a model with check scripts
    When "validate" is run with and without the scripts present
    Then its output is identical and it does not run the checks
```
