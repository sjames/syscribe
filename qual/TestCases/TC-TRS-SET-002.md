---
id: TC-TRS-SET-002
type: TestCase
testLevel: L3
status: draft
name: "Verify achieves.add and evidence.add validate their target before writing and append without disturbing existing order."
verifies:
  - REQ-TRS-SET-002
---

```gherkin
Feature: set achieves.add / evidence.add
  Scenario: achieves.add refuses a dangling target
    Given a PlanningItem
    When "set <id> achieves.add REQ-NONEXISTENT" is run
    Then the command exits non-zero and the file is unchanged

  Scenario: achieves.add refuses a non-Requirement target
    Given a PlanningItem and an unrelated PartDef
    When "set <id> achieves.add <PartDef>" is run
    Then the command exits non-zero, naming that it must resolve to a native Requirement

  Scenario: achieves.add appends after existing entries
    Given a PlanningItem that already achieves one Requirement
    When "set <id> achieves.add <second-req-id>" is run
    Then the second requirement appears after the first, in that order

  Scenario: evidence.add ref must resolve
    Given a PlanningItem with no evidence
    When "set <id> evidence.add ref=NONEXISTENT" is run
    Then the command exits non-zero and the file is unchanged

  Scenario: evidence.add path must exist on disk or be a remote URI
    Given a PlanningItem
    When "set <id> evidence.add path=<nonexistent-local-path>" is run
    Then the command exits non-zero
    When "set <id> evidence.add path=<existing-local-path>" or an http(s):// URI is run
    Then the command succeeds

  Scenario: achieves.add of an already-present id is a no-op
    Given a PlanningItem that already achieves a Requirement
    When "set <id> achieves.add <that-req-id>" is run
    Then the command reports that the id is already present, the id is not duplicated, and the file is unchanged

  Scenario: evidence.add refuses both ref and path, or neither
    Given a PlanningItem with no evidence
    When "set <id> evidence.add ref=<id> path=<path>" is run
    Then the command exits non-zero and the file is unchanged
    When "set <id> evidence.add" is run with neither ref= nor path=
    Then the command exits non-zero and the file is unchanged

  Scenario: evidence.add carries an optional rationale onto the new entry
    Given a PlanningItem with no evidence
    When "set <id> evidence.add ref=<id> rationale=<text>" is run
    Then the new evidence entry carries both the ref and that rationale text
    When "set <id> evidence.add path=<path>" is run with no rationale=
    Then the new evidence entry carries no rationale

  Scenario: --dry-run previews achieves.add and evidence.add without writing
    Given a PlanningItem
    When "set <id> achieves.add <req-id> --dry-run" is run
    Then a unified diff adding the requirement is printed and the file is unchanged
    When "set <id> evidence.add path=<path> --dry-run" is run
    Then a unified diff adding the evidence entry is printed and the file is unchanged
```
