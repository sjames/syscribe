---
type: TestCase
id: TC-TRS-SEQ-002
name: "A call of an operation is a message with its signature, and a return"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
verifies:
  - REQ-TRS-SEQ-002
tags:
  - diagram
  - sequence
---

```gherkin
Feature: A call of an operation is a message with its signature, and a return

  Scenario: call and return
    Then a synchronous call is a labelled message followed by a return

  Scenario: asynchronous
    Then an asynchronous call has no return

  Scenario: participant
    Then the operation's owner is a lifeline unless to or via names one
```
