---
type: TestCase
id: TC-TRS-PISTEP-001
name: "steps renders the child-item breakdown"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_questions.rs
verifies:
  - REQ-TRS-PISTEP-001
tags:
  - planning
---

```gherkin
Feature: Steps view

  Scenario: tree and roll-up
    Then children are shown recursively with status and a leaf roll-up, with --open, --depth and --json

  Scenario: hints only
    Then disagreement between a parent and its children is noted and validate stays silent
```
