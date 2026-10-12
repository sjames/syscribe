---
id: TC-TRS-PROJ-009
type: TestCase
testLevel: L3
status: draft
name: "Verify a machine parameter with no default is one W248 rather than a parse error, and --config checks the machine at the bound value."
verifies:
  - REQ-TRS-PROJ-009
---

```gherkin
Feature: machine parameters without a default
  Scenario: base model
    Given machines M and N whose expressions use Features::Profile.limit, which has no default
    When the tool runs `validate`
    Then exactly one W248 names Features::Profile.limit, both machines and --config
    And no E882, E884 or W933 is reported
  Scenario: bound value under --config
    Given a configuration that binds limit to a value that is not an expression operand
    When the tool runs `validate --config` on it
    Then W933 names the bound value
  Scenario: each configuration at its own bindings
    Given two configurations that select the same features, one binding a value that is not an expression operand
    When the tool runs `validate --all-configs`
    Then only that configuration's row counts the W933 findings
  Scenario: parameter with a default
    Given the same machines and a default on limit
    When the tool runs `validate`
    Then no W248, E882, E884 or W933 is reported
```
