---
type: TestCase
id: TC-TRS-HELPDISC-001
name: "help answers by type, keyword and task without a model"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/help_discovery_and_hints.rs
verifies:
  - REQ-TRS-HELPDISC-001
tags:
  - cli
---

```gherkin
Feature: Help discovery

  Scenario: help for a type
    Then help PlanningItem lists claim, release and the status vocabulary, with no model

  Scenario: keyword search
    Then help --search evidence returns set, ingest-results and results with the matching line

  Scenario: tasks and unknown topics
    Then help tasks maps jobs to commands and a mistyped topic lists the nearest commands

  Scenario: template without a model
    Then template prints its skeleton when the model directory does not exist
```
