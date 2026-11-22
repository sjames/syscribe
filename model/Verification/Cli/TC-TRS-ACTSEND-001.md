---
type: TestCase
id: TC-TRS-ACTSEND-001
name: "a SendAction inside a called activity reaches the machine's outbox and the connected part"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/comp_stress.rs
verifies:
  - REQ-TRS-ACTSEND-001
tags:
  - action-flow
  - state-machine
  - composition
---

```gherkin
Feature: activities emit messages for the machine that runs them

  Scenario: an activity with a SendAction and nothing else is executable
    Given an activity with a SendAction but no variables or assign
    Then the machine runs it and emits the message

  Scenario: a send in a transition effect activity
    Given a machine whose transition effect names an activity with one SendAction
    When the transition is taken
    Then the machine's outbox holds one message with the event, port and payload values the SendAction evaluated

  Scenario: ordering and the trace
    Given an entry activity that sends twice and a machine-level send in the same microstep
    Then the messages appear in firing order and the step's actions list shows each as "send ..."

  Scenario: nested and conditional sends
    Given an activity that sends inside a loop body, a performed callee and an if branch
    Then one message is emitted per send that fired, none for an untaken branch

  Scenario: delivery across a connection
    Given two connected parts where the sender's activity sends through its out port
    Then the receiver's machine reacts to the message, with the connection's delay

  Scenario: failure discards the activity's sends
    Given an activity that sends and then fails, or one that waits for an event
    Then the run stops with the error and no message of that activity is emitted
```
