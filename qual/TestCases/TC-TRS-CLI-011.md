---
id: TC-TRS-CLI-011
type: TestCase
testLevel: L3
status: draft
name: "Verify list matches every element type by its canonical name, including Zone, Conduit and TestPlan."
verifies:
  - REQ-TRS-CLI-011
---

```gherkin
Feature: list covers every element type (TC-TRS-CLI-011)

  Scenario: list finds Zone, Conduit and TestPlan elements
    Given a model with a Zone, a Conduit and a TestPlan
    When the tool runs list Zone, list Conduit and list TestPlan
    Then each lists its element
    And show labels the Zone with its type name

  Scenario: list finds an element of every type in the inventory
    Given a template skeleton of every element type rendered into one model
    When the tool runs list for each type in the inventory
    Then each lists at least one element
```
