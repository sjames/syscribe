---
id: TC-TRS-SET-001
type: TestCase
testLevel: L3
status: draft
name: "Verify set status=<value> validates per-type enum, splices byte-preservingly, cross-checks PlanningItem done against W310, and supports --dry-run."
verifies:
  - REQ-TRS-SET-001
---

```gherkin
Feature: set status=<value>
  Scenario: an out-of-enum status value is refused and writes nothing
    Given a Requirement at status: draft
    When "set <id> status=bogus" is run
    Then the command exits non-zero, names the allowed values, and the file is unchanged

  Scenario: a valid status value writes only that line
    Given a Requirement with a quoted name: field and status: draft
    When "set <id> status=approved" is run
    Then the status: line is updated and every other line, including the name: field's quoting, is unchanged

  Scenario: the target resolves by qualified name or stable id
    Given a Requirement reachable by both its qualified name and its stable id
    When "set" is run with either form
    Then the command succeeds either way

  Scenario: marking a PlanningItem done warns on an under-verified achieves requirement but still writes
    Given a PlanningItem that achieves a Requirement with no active TestCase
    When "set <PI-id> status=done" is run
    Then a warning naming the achieves Requirement is printed and the status is still written

  Scenario: --dry-run previews without writing
    Given a Requirement at status: draft
    When "set <id> status=approved --dry-run" is run
    Then a unified diff is printed and the file is unchanged
```
