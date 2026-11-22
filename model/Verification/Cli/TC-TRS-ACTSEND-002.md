---
type: TestCase
id: TC-TRS-ACTSEND-002
name: "a sends-only effect activity sends, and a composite sees its sends"
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

  Scenario: a sends-only activity delivers its message
    Given a transmitter whose transition effect names an ActionDef of one SendAction and nothing else
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
```
