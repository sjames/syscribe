---
id: TC-BAD-001
type: TestCase
testLevel: L3
status: approved
title: "Out-of-scope broken test"
verifies: [REQ-GHOST-999]
---
```gherkin
Feature: TC-BAD-001
Scenario: nominal
  Given x
  When y
  Then z shall hold
```
