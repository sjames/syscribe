---
id: TC-TRS-VAL-011
type: TestCase
testLevel: L3
status: draft
name: "Verify actionable E106 messages and scaffold-gherkin --fix alignment."
verifies:
  - REQ-TRS-VAL-011
---

Verify that `E106` carries the exact `Scenario:` fix text, and that `scaffold-gherkin --fix` inserts the missing scenario so `E106` clears.

```gherkin
Feature: Actionable findings and Gherkin scaffolding

  Scenario: E106 message is actionable
    Given a TestCase whose testFunctions[].scenario has no matching Scenario title
    When the tool is invoked
    Then the E106 message contains the exact "Scenario:" line to add

  Scenario: scaffold-gherkin --fix aligns the Gherkin block
    Given a TestCase missing one Scenario block
    When scaffold-gherkin --fix is run on a working copy
    Then the missing Scenario is inserted and E106 no longer fires
```
