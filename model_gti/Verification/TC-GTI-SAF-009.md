---
type: TestCase
id: TC-GTI-SAF-009
name: "Sensing common cause: perturbation-response check"
status: active
testLevel: L5
subject: Behavior::Chain::ProtectionChain
verifies:
  - REQ-GTI-SYS-045
  - REQ-GTI-SW-008
tags: [executable, reg-unverified]
---

Executable scenarios of the perturbation-response check in the protection chain, for either region: run with `syscribe simulate TC-GTI-SAF-009 --config CONF-GTI-US-M3-001` (or an India configuration). The grid's answer to the injected current comes from the in-process plant `Behavior::Plants::GridImpedance`. The island-clearing time of both profiles is 2 s. `Breaker` opens the grid breaker with a balanced local load, so the voltage and frequency the sensors see do not change: the classic case the voltage/frequency window cannot detect. The region-specific case of a grid event during a measurement is in TC-GTI-PROT-US-001 and TC-GTI-PROT-IN-001. The current-sensing path itself (`REQ-GTI-HW-010`) is not modelled, so the check reads the injected current as commanded; TC-GTI-SAF-010 verifies it on the bench. With both voltage chains stuck (`FTE-GTI-009`) the probe reads no response at all, so the check trips on a healthy grid too: the inverter fails safe, and the scenarios do not claim it told an island from a stuck sensor.

```gherkin
Feature: Both voltage chains stuck at nominal

  Background:
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal

  Scenario: an islanded grid is cleared although the voltage and frequency stay inside the window
    When grid receives Breaker
    And 2 s elapses
    Then monitor is in state Normal
    And supervisor is in state Tripped
    And powerStage.pwm == 0 and relayA.closed == 0 and relayB.closed == 0

  Scenario: grid lost with both sensing chains reading nominal
    Given fault FTE-GTI-009 is injected
    When grid receives Breaker
    And 2 s elapses
    Then monitor is in state Normal
    And hwTrip is in state Armed
    And supervisor is in state Tripped
    And powerStage.pwm == 0 and relayA.closed == 0 and relayB.closed == 0

  Scenario: both chains stuck on a healthy grid trip the inverter as well (fail-safe, not island detection)
    Given fault FTE-GTI-009 is injected
    When 2 s elapses
    Then monitor is in state Normal
    And supervisor is in state Tripped
    And powerStage.pwm == 0 and relayA.closed == 0

  Scenario: healthy grid is not tripped
    When 60 s elapses
    Then supervisor is in state Running
    And perturb sent SampleReq via req

  Scenario: weak grid
    When grid receives Impedance with z = 8 SI::ohm
    And 2 s elapses
    Then supervisor is in state Tripped
    And powerStage.pwm == 0 and relayA.closed == 0

  Scenario: an islanded grid is cleared by the check alone when the monitor and the hardware trip are both dead
    Given fault FTE-GTI-004 is injected
    And fault FTE-GTI-006 is injected
    When grid receives Breaker
    And 2 s elapses
    Then supervisor is in state Tripped
    And powerStage.pwm == 0 and relayA.closed == 0
```
