---
id: TC-TRS-MG-010
type: TestCase
testLevel: L3
status: draft
name: "Verify refines on behavioral defs: ActionDef/StateDef refine resolves + refinedBy; E316 on bad target; no W307 on a behavioral def."
verifies:
  - REQ-TRS-MG-010
---

```gherkin
Feature: refines honored on behavioral definitions
  Scenario: an ActionDef refines link resolves and back-links the requirement
    Given an ActionDef whose refines names a Requirement by id
    When the model is validated
    Then the target resolves and the requirement reports the action under refinedBy in show

  Scenario: resolution by qualified name also works
    Given an ActionDef whose refines names a Requirement by qualified name
    When the model is validated
    Then the requirement reports the action under refinedBy

  Scenario: an unresolved behavioral refines target raises E316
    Given a StateDef whose refines names an operand that resolves to nothing
    When the model is validated
    Then E316 is raised against the state

  Scenario: a behavioral refines target that is not a requirement raises E316
    Given an ActionDef whose refines names a PartDef
    When the model is validated
    Then E316 is raised naming the resolved type

  Scenario: a behavioral def with no refines raises no W307
    Given an ActionDef with no refines link
    When the model is validated
    Then no W307 finding is produced for the action
```
