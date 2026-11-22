---
type: TestCase
id: TC-TRS-ACTKEYS-001
name: "ignored sub-action keys and payload forms are reported and the working form is named"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/effect_sends.rs
verifies:
  - REQ-TRS-ACTKEYS-001
tags:
  - action-flow
  - validation
---

```gherkin
Feature: W938 on sub-action keys

  Scenario: payload forms that are ignored name the working one
    Given a SendAction with arguments, values or a mapping as payload
    When the model is validated
    Then W938 is raised and names parameters as the way to bind payload fields

  Scenario: the working form is clean
    Given a SendAction with parameters binding a payload field
    When the model is validated
    Then no W938 is raised

  Scenario: any kind is checked
    Given an AssignmentAction with an unknown key
    When the model is validated
    Then W938 names the key
```
