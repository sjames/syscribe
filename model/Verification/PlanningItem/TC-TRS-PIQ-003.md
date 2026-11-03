---
type: TestCase
id: TC-TRS-PIQ-003
name: "set adds, answers and defers questions"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_questions.rs
verifies:
  - REQ-TRS-PIQ-003
tags:
  - planning
---

```gherkin
Feature: Writing questions

  Scenario: add answer defer
    Then each edits only its own entry, keeping comments and order

  Scenario: refusals
    Then a non-PlanningItem, an unknown Qn or empty text exit 1 and write nothing
```
