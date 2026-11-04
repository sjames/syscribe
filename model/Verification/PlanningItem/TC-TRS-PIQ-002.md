---
type: TestCase
id: TC-TRS-PIQ-002
name: "questions, list and show surface open questions"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_questions.rs
verifies:
  - REQ-TRS-PIQ-002
tags:
  - planning
---

```gherkin
Feature: Surfacing questions

  Scenario: questions command
    Then it lists every question with its owning item and filters by --open, --status and --item

  Scenario: list and show
    Then list --open-questions keeps items with an open question and show prints them

  Scenario: prose table hint
    Then validate --hints raises I913 for a Questions heading with an empty questions list
```
