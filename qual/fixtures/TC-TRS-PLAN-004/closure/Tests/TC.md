---
id: TC-PL4-LEAF-001
type: TestCase
testLevel: L3
status: approved
title: "Leaf test"
verifies: [REQ-PL4-LEAF-001]
---
```gherkin
Feature: TC-PL4-LEAF-001
Scenario: nominal
  Given the leaf
  When exercised
  Then it shall hold
```
