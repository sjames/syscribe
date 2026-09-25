---
id: TC-TRS-FTA-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that FaultTreeEvent ref: is accepted, resolved (E927 on dangling) and surfaced by show, links and fault-tree render."
verifies:
  - REQ-TRS-FTA-002
---

Verify that a `FaultTreeEvent`'s `ref:` link to the architecture element it models is a recognised, resolved and surfaced schema field.

```gherkin
Feature: FaultTreeEvent ref link

  Scenario: A resolving ref is accepted without W047 or E927
    Given a fault tree whose events declare ref: by qualified name and by id, both resolving
    When the tool validates the model
    Then no W047 and no E927 finding is emitted

  Scenario: A dangling ref raises E927
    Given a fault tree event whose ref: names a non-existent element
    When the tool validates the model
    Then an E927 finding is emitted naming the unresolved target

  Scenario: The link is surfaced
    Given the resolving fixture
    When show, links and fault-tree render are invoked
    Then show lists the ref, links lists the event as an inbound ref source and the id-authored ref as a resolved outbound, and the rendered node label names the referenced element
```
