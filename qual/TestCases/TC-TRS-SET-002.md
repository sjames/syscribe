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
```
