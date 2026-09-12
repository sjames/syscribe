---
id: TC-TRS-PLANITEM-011
type: TestCase
testLevel: L3
status: draft
name: "Verify claim/release set and clear claimedBy/claimedAt correctly, refuse on conflicting/done targets, and surface claimedBy in show/list --json."
verifies:
  - REQ-TRS-PLANITEM-011
---

```gherkin
Feature: PlanningItem claim/release
  Scenario: claim sets claimedBy and claimedAt
    Given an unclaimed PlanningItem
    When "claim <id> --by agent-1" is run
    Then claimedBy and claimedAt are set on the file

  Scenario: claim refuses when already claimed by someone else
    Given a PlanningItem claimed by agent-1
    When "claim <id> --by agent-2" is run
    Then the command exits non-zero, names the current claimant, and the file is unchanged

  Scenario: re-claiming with the same agent is allowed
    Given a PlanningItem claimed by agent-1
    When "claim <id> --by agent-1" is run again
    Then the command succeeds

  Scenario: claim refuses on an already-done item
    Given a PlanningItem at status: done
    When "claim <id> --by agent-1" is run
    Then the command exits non-zero, saying there is nothing to claim, and the file is unchanged

  Scenario: release clears both fields regardless of status
    Given a claimed PlanningItem at status: blocked
    When "release <id>" is run
    Then claimedBy and claimedAt are both removed and status is unchanged

  Scenario: release on an unclaimed item is a no-op
    Given an unclaimed PlanningItem
    When "release <id>" is run
    Then nothing is written and a "nothing to release" message is printed

  Scenario: claimedBy is visible in show and list --json
    Given a claimed PlanningItem
    When "show <id>" and "list PlanningItem --json" are run
    Then both surface the claimedBy value

  Scenario: --dry-run previews without writing for both commands
    Given a PlanningItem
    When "claim <id> --by agent-1 --dry-run" and "release <id> --dry-run" are run
    Then each prints a diff and neither writes to disk
```
