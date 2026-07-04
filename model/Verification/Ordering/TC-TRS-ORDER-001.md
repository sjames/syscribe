---
type: TestCase
id: TC-TRS-ORDER-001
name: "displayOrder governs requirements-report order with unset sinking last and id tie-break"
status: draft
testLevel: L2
tags:
  - ordering
  - authoring
verifies:
  - REQ-TRS-ORDER-001
---

Verifies that the numeric `displayOrder` frontmatter field controls the presentation
order of requirements in the Markdown validation report: elements sort by ascending
`displayOrder`, an element without `displayOrder` sorts after every element that has
one, and equal or absent values fall back to stable-identifier order. Also confirms a
decimal value slots correctly between two integer neighbours.

```gherkin
Feature: displayOrder controls requirement presentation order

  Scenario: Requirements report honours ascending displayOrder before identifier order
    Given a model with requirements REQ-A-003 (displayOrder 30), REQ-A-001 (displayOrder 10),
      and REQ-A-002 (displayOrder 20)
    When syscribe -m <root> is run
    Then the Requirements table lists REQ-A-001 before REQ-A-002 before REQ-A-003

  Scenario: A decimal displayOrder inserts between integer neighbours
    Given the model additionally contains REQ-A-004 with displayOrder 15
    When syscribe -m <root> is run
    Then the Requirements table lists REQ-A-004 between REQ-A-001 and REQ-A-002

  Scenario: Requirements without displayOrder sort after ordered ones, by identifier
    Given the model additionally contains REQ-A-009 and REQ-A-008 with no displayOrder
    When syscribe -m <root> is run
    Then REQ-A-008 and REQ-A-009 appear after every requirement that declares displayOrder
    And REQ-A-008 appears before REQ-A-009
```
