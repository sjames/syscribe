---
type: TestCase
id: TC-TRS-SMEXEC-001
name: "the state-machine interpreter follows SCXML run-to-completion with SysML v2 change and time triggers"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/sm_run.rs
verifies:
  - REQ-TRS-SMEXEC-001
tags:
  - state-machine
---

```gherkin
Feature: state-machine interpreter

  Scenario: start and ordering
    Then the initial state is entered with its entry actions, and a transition runs exit actions, then its effect, then entry actions

  Scenario: selection
    Then a deeper state preempts its ancestor, a false guard leaves the event unhandled, and unhandled events are ignored or an error in strict mode

  Scenario: time
    Then an after timer fires when the virtual clock reaches it, is cancelled on exit, and a zero delay fires on entry

  Scenario: change triggers
    Then a when trigger fires on false to true, at once when already true on entry, once per arming, and is consumed even if its guard fails

  Scenario: composite and parallel
    Then done triggers follow final states, internal transitions neither exit nor enter, and parallel regions react together

  Scenario: errors and determinism
    Then an unbound name or dimension error stops the run, a livelock hits the microstep bound, and a re-run gives an identical trace
```
