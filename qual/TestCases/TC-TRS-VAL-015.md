---
id: TC-TRS-VAL-015
type: TestCase
testLevel: L3
status: draft
name: "Verify informational I010 for planned TestCase sources: emitted for draft, deniable, exit-neutral, none for retired."
verifies:
  - REQ-TRS-VAL-015
---

Verify that a non-active TestCase whose sources are not yet present emits informational `I010` instead of `W004`/`W009`, that `I010` does not by itself fail validation but is selectable via `--deny`, and that `retired` TestCases emit nothing.

```gherkin
Feature: Informational I010 for planned TestCase sources

  Scenario: Draft TestCase with a missing source emits I010, not W004
    Given a draft TestCase whose sourceFile does not exist
    When the tool is invoked
    Then I010 is emitted for it and W004 is not

  Scenario: I010 alone does not fail validation
    Given a model whose only drift-related finding is I010
    When validate is invoked
    Then the exit code is 0

  Scenario: I010 can be gated explicitly
    Given the same model
    When validate is invoked with --deny I010
    Then the exit code is 2

  Scenario: Retired TestCases emit nothing
    Given a retired TestCase whose sourceFile does not exist
    When the tool is invoked
    Then neither W004, W009, nor I010 is emitted for it
```
