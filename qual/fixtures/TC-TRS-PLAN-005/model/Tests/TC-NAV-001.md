---
id: TC-NAV-001
type: TestCase
testLevel: L3
status: approved
title: "Waypoint hold test"
verifies: [REQ-NAV-002]
testFunctions:
  - function: "nav::tests::pass_nav"
    scenario: "nominal"
---
```gherkin
Feature: TC-NAV-001
Scenario: nominal
  Given a waypoint
  When commanded
  Then it shall hold
```
