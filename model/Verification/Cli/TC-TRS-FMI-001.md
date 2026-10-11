---
type: TestCase
id: TC-TRS-FMI-001
name: "FMU plants run with state machines on the shared virtual clock, only when the fmi feature is built"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-fmi/tests/plant.rs
verifies:
  - REQ-TRS-FMI-001
tags:
  - fmi
---

```gherkin
Feature: FMI plant co-simulation

  Scenario: gating
    Then without the fmi feature the fields parse, I011 is raised and simulate of a plant TestCase exits 1, and with it the checks run

  Scenario: plant declaration and wiring
    Then E888, E889 and E966 are raised for malformed declarations, wrong-direction or wrong-unit connections and unreadable, mismatched or tampered FMUs

  Scenario: the loop
    Then a machine reading BouncingBall's height fires its when trigger at the first step boundary after the crossing, with the reference numbers for FMI 2.0 and 3.0

  Scenario: timers cut the step
    Then a discrete timer between step boundaries fires at its exact due time

  Scenario: units
    Then values cross the boundary in SI base units and a dimension mismatch is rejected before the run

  Scenario: evidence
    Then the recorded provenance names the FMU hash, FMI version, step and platform, and a changed FMU is detected
```
