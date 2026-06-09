---
id: TC-PL2-ESC-001
type: TestCase
title: "Adas-only test"
status: approved
testLevel: L3
verifies: [REQ-PL2-001]
appliesWhen: Features::Fa
---
```gherkin
Feature: TC-PL2-ESC-001
Scenario: nominal
  Given Fa is enabled
  When exercised
  Then it shall behave correctly
```
