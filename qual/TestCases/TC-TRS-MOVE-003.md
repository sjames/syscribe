---
id: TC-TRS-MOVE-003
type: TestCase
testLevel: L3
status: draft
name: "Verify move is atomic — a failing precondition leaves the model unchanged."
verifies:
  - REQ-TRS-MOVE-003
---

Verify that when a move fails a precondition, no file on disk is modified, created, or removed (all-or-nothing).

```gherkin
Feature: Move is atomic

  Scenario: A rejected move leaves every file byte-for-byte unchanged
    Given a recorded checksum of every file in the model
    When a move whose destination already exists is attempted
    Then the command exits non-zero
    And every file's checksum is identical to before

  Scenario: --dry-run never writes
    Given a valid move
    When move ... --dry-run is run
    Then the planned changes are reported and no file is modified
```
