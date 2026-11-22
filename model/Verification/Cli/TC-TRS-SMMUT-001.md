---
type: TestCase
id: TC-TRS-SMMUT-001
name: "mutate seeds faults into the machine IR and scores how many the scenarios detect"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/sm_mut.rs
verifies:
  - REQ-TRS-SMMUT-001
tags:
  - state-machine
---

```gherkin
Feature: mutation analysis of scenarios

  Scenario: thorough scenarios
    Then every mutant of every operator is killed and the score is 100

  Scenario: weak scenarios
    Then survivors are listed with the mutation applied and its location

  Scenario: equivalent suspects
    Then uncovered mutants with identical traces are reported apart and do not lower the score

  Scenario: gate and preconditions
    Then --min-score decides the exit code and scenarios must pass on the unmutated machine

  Scenario: the loss of a send inside an effect activity is seeded
    Given a composite whose transmitter sends through an effect ActionDef
    When mutate runs
    Then a remove-send mutant names the send, the activity and the transition that runs it
    And a scenario that checks the message kills it while one that never checks it lets it survive
```
