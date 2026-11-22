---
type: TestCase
id: TC-TRS-SYSCOMP-002
name: "a plain accept receives a port message when the part has exactly one in port"
status: active
testLevel: L3
sourceFile: repo:crates/syscribe/tests/plain_accept.rs
verifies:
  - REQ-TRS-SYSCOMP-001
tags:
  - composition
  - state-machine
---

```gherkin
Feature: plain accepts inside a composite (GH #513)

  Scenario: a plain accept receives from the part's only in port
    Given a receiver machine that accepts Cmd with no via, on a part with exactly one in port
    When the transmitter sends Cmd to it
    Then the receiver takes the transition and W936 is not raised

  Scenario: an explicit via accept is unchanged
    Given the receiver accepts Cmd via the port
    Then it takes the transition as before

  Scenario: several in ports need an explicit via
    Given the same receiver on a part with two in ports
    Then a plain accept receives nothing and W936 says to name the port with via

  Scenario: the machine is not edited for the composite
    Given the receiver machine used alone
    When the plain event is injected
    Then it takes the transition
```
