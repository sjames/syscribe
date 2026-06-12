---
id: TC-TRS-MOVE-004
type: TestCase
testLevel: L3
status: draft
name: "Verify move preserves stable IDs and references made through them."
verifies:
  - REQ-TRS-MOVE-004
---

Verify that moving a native `Requirement`/`TestCase` keeps its stable `id` unchanged and does not rewrite references expressed through that id.

```gherkin
Feature: Move preserves stable identifiers

  Scenario: Stable id is unchanged after a move
    Given a native Requirement REQ-MV-001 at some qualified name
    When the requirement file is moved to a new qualified name
    Then its id is still REQ-MV-001

  Scenario: Id-based references are not rewritten and still resolve
    Given a TestCase that verifies REQ-MV-001 by its stable id
    When REQ-MV-001 is moved
    Then the verifies entry still reads REQ-MV-001 and still resolves
```
