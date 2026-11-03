---
type: TestCase
id: TC-TRS-PIQ-001
name: "Questions are validated"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_questions.rs
verifies:
  - REQ-TRS-PIQ-001
tags:
  - planning
---

```gherkin
Feature: Inline questions

  Scenario: malformed entries
    Then E724 E725 E726 E727 are raised for a bad entry, a duplicate id, a bad status and an answer-less answered question

  Scenario: done with an open question
    Then W317 is raised, and not for deferred or answered questions

  Scenario: questions on another type
    Then W318 is raised on a non-PlanningItem that carries the field, and not on a PlanningItem

  Scenario: no questions
    Then an item without the field is unaffected
```
