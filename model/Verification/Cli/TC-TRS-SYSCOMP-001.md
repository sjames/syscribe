---
type: TestCase
id: TC-TRS-SYSCOMP-001
name: "The parts of a composite execute together on one clock, communicating over connections and bindings"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/system_run.rs
verifies:
  - REQ-TRS-SYSCOMP-001
tags:
  - composition
---

```gherkin
Feature: System composition

  Scenario: machine output
    Then a send effect lands in the outbox with its payload evaluated at send time, and machines without send keep their IR and hash

  Scenario: static checks
    Then E930 to E934 and W936 are raised for composites with unresolved or ambiguous behaviours, wrong connections, unknown ports, bad bindings and unhandled traffic, and a composite without executable machines is left alone

  Scenario: messages and time
    Then a message is delivered with its payload, a connection delay postpones delivery to exactly that instant, fan-out follows connection order and a livelock ends in an error

  Scenario: bindings
    Then bound values follow their source into the sink machine

  Scenario: scenarios
    Then part-qualified steps run on the composite through simulate, failures name the step, and the messages and the hash are reported
```
