---
type: TestCase
id: TC-TRS-REQTRACE-001
name: "the trace function returns shortest-path nodes and edges to a category, and jump links are fixed paths"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQTRACE-001
tags:
  - server
---

```gherkin
Feature: explorer trace highlighting

  Scenario: shortest paths to a category
    Given a graph with two routes of different length to a TestCase
    Then only the shorter route is returned, in either edge direction, and the start is never a target

  Scenario: nothing matches
    Then the result is empty

  Scenario: jump links
    Then Feature, Configuration, FeatureModel and PlanningItem nodes get fixed paths and other kinds none
```
