---
id: TC-MV-001
type: TestCase
title: "Verify widget"
status: draft
testLevel: L3
verifies:
  - REQ-MV-001
---
```gherkin
Feature: widget
  Scenario: present
    Given a widget
    Then it exists
```
