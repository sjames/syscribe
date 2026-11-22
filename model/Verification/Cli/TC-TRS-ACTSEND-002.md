---
type: TestCase
id: TC-TRS-ACTSEND-002
name: "a composite sees effect-activity sends, and a silent sends-only activity is explained"
status: active
testLevel: L3
sourceFile: repo:crates/syscribe/tests/effect_sends.rs
verifies:
  - REQ-TRS-ACTSEND-001
tags:
  - action-flow
  - composition
---

```gherkin
Feature: effect activities that send (GH #509, #516)

  Scenario: an opted-in sends-only activity delivers its message
    Given a transmitter whose transition effect names an ActionDef of one SendAction with variables: []
    When tx receives Tick and 6 ms elapse
    Then the receiver reaches its final state

  Scenario: W936 follows sends inside effect activities
    Given a receiver that accepts a message sent only by an effect activity of the connected part
    When the model is validated
    Then no W936 is raised for that accept

  Scenario: W936 still fires when no activity sends
    Given the same composite whose effect activity only assigns
    When the model is validated
    Then W936 is raised

  Scenario: a sends-only activity that has not opted in is explained
    Given an effect ActionDef with a SendAction and no variables or assignment
    When the model is validated
    Then W937 names the action and the variables: [] opt-in
    And simulate sends nothing, as REQ-TRS-ACTSEND-001 states
```
