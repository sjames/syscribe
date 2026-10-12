---
type: TestCase
id: TC-GTI-PROT-US-001
name: "Protection chain behaviour, US profile"
status: active
testLevel: L4
subject: Behavior::Chain::ProtectionChain
verifies:
  - REQ-GTI-SYS-007
  - REQ-GTI-SYS-010
  - REQ-GTI-SYS-030
  - REQ-GTI-SYS-038
  - REQ-GTI-SYS-039
  - REQ-GTI-SW-001
  - REQ-GTI-SW-008
  - REQ-GTI-HW-008
  - REQ-GTI-HW-009
tags: [executable, region-us, reg-unverified]
appliesWhen: Features::Region::US
---

Executable scenarios of the protection chain with the US grid profile. Run with `syscribe simulate TC-GTI-PROT-US-001 --config CONF-GTI-US-M3-001`. Limits are the unverified values of the regulatory matrix.

```gherkin
Feature: Grid connection and protection chain (US)

  @verifies REQ-GTI-SYS-010
  Scenario: the inverter connects after the reconnect delay and ramps up
    When grid receives Set with v = 1.0, f = 60
    And supervisor receives Start
    Then supervisor is in state WaitGrid
    When 299 s elapses
    Then supervisor is in state WaitGrid
    And relayA.closed == 0
    And powerStage.pwm == 0
    When 1 s elapses
    Then supervisor is in state Closing
    And relayA is in state Closed
    And relayB is in state Closed
    And relayA.closed == 1 and relayB.closed == 1
    And powerStage.pwm == 0
    When 99 ms elapses
    Then supervisor is in state Closing
    When 1 ms elapses
    Then supervisor is in state Ramping
    And powerStage is in state On
    And powerStage.pwm == 1
    When 4 s elapses
    Then supervisor is in state Ramping
    When 1 s elapses
    Then supervisor is in state Running

  @verifies REQ-GTI-SYS-030
  Scenario: insulation below the limit prevents connection until it recovers
    Given supervisor.rIso is 20
    When grid receives Set with v = 1.0, f = 60
    And supervisor receives Start
    Then supervisor is in state InsulationFault
    When 310 s elapses
    Then relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0
    When supervisor.rIso becomes 4000
    And supervisor receives Start
    Then supervisor is in state WaitGrid

  @verifies REQ-GTI-SYS-030
  Scenario Outline: the insulation limit is 34 kOhm from standby
    Given supervisor.rIso is <r>
    When grid receives Set with v = 1.0, f = 60
    And supervisor receives Start
    Then supervisor is in state <state>

    Examples:
      | r | state |
      | 33 | InsulationFault |
      | 34 | WaitGrid |
      | 35 | WaitGrid |

  @verifies REQ-GTI-SYS-030
  Scenario Outline: the insulation limit is 34 kOhm after a fault
    Given supervisor is in state InsulationFault
    And supervisor.rIso is <r>
    And supervisor.gridOk is 1
    When supervisor receives Start
    Then supervisor is in state <state>

    Examples:
      | r | state |
      | 33 | InsulationFault |
      | 34 | WaitGrid |
      | 35 | WaitGrid |

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario: a persistent high-frequency excursion trips after the clearing time
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 1.0, f = 62.5
    And 159 ms elapses
    Then supervisor is in state Running
    When 1 ms elapses
    Then supervisor is in state Tripped
    And relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario: a persistent low-frequency excursion also trips
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 1.0, f = 56.0
    And 160 ms elapses
    Then supervisor is in state Tripped
    And powerStage.pwm == 0

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario: an excursion shorter than the clearing time does not trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 1.0, f = 62.5
    And 80 ms elapses
    And grid receives Set with v = 1.0, f = 60
    And 1 s elapses
    Then supervisor is in state Running
    And powerStage.pwm == 1

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario: voltage inside the window does not trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0.5, f = 60
    And 1 s elapses
    Then supervisor is in state Running

  @verifies REQ-GTI-SYS-038,REQ-GTI-SYS-039,REQ-GTI-HW-008,REQ-GTI-HW-009
  Scenario: grid loss disconnects at once through the hardware trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    Then relayB.closed == 0
    And powerStage.pwm == 0
    When 160 ms elapses
    Then supervisor is in state Tripped
    And relayA.closed == 0

  @verifies REQ-GTI-SYS-010
  Scenario: the inverter reconnects only after the grid is back and the delay has passed
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    And 160 ms elapses
    And grid receives Set with v = 1.0, f = 60
    Then supervisor is in state WaitGrid
    When 299 s elapses
    Then relayA.closed == 0
    When 1 s elapses
    Then relayA.closed == 1

  @verifies REQ-GTI-SYS-007,REQ-GTI-SYS-038
  Scenario Outline: grid loss in any connection phase aborts the connection
    Given supervisor is in state <phase>
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    When grid receives Set with v = 1.0, f = 62.5
    And 160 ms elapses
    Then supervisor is in state Tripped
    And supervisor.gridOk == 0
    And powerStage.pwm == 0
    And powerStage is in state Off
    And relayA is in state Open
    And relayB is in state Open
    And relayA.closed == 0 and relayB.closed == 0

    Examples:
      | phase    |
      | WaitGrid |
      | Closing  |
      | Ramping  |

  @verifies REQ-GTI-SYS-039,REQ-GTI-HW-008,REQ-GTI-HW-009
  Scenario: the hardware trip re-arms by itself when the voltage returns
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    Then hwTrip is in state Latched
    When grid receives Set with v = 0.3, f = 60
    Then hwTrip is in state Latched
    When grid receives Set with v = 1.0, f = 60
    Then hwTrip is in state Armed
    And relayB.closed == 0

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor classifies samples at the window boundaries
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.2 | 60 | Normal |
      | 1.21 | 60 | Excursion |
      | 0.45 | 60 | Normal |
      | 0.44 | 60 | Excursion |
      | 1.0 | 62 | Normal |
      | 1.0 | 62.1 | Excursion |
      | 1.0 | 56.5 | Normal |
      | 1.0 | 56.4 | Excursion |

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor classifies samples at the window boundaries when normal
    Given monitor is in state Normal
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.2 | 60 | Normal |
      | 1.21 | 60 | Excursion |
      | 0.45 | 60 | Normal |
      | 0.44 | 60 | Excursion |
      | 1.0 | 62 | Normal |
      | 1.0 | 62.1 | Excursion |
      | 1.0 | 56.5 | Normal |
      | 1.0 | 56.4 | Excursion |

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor recovers from Excursion exactly at the window edge
    Given monitor is in state Excursion
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.2 | 60 | Normal |
      | 1.21 | 60 | Excursion |
      | 0.45 | 60 | Normal |
      | 0.44 | 60 | Excursion |
      | 1.0 | 62 | Normal |
      | 1.0 | 62.1 | Excursion |
      | 1.0 | 56.5 | Normal |
      | 1.0 | 56.4 | Excursion |

  @verifies REQ-GTI-SYS-010,REQ-GTI-SW-001
  Scenario Outline: the monitor reports the grid good again only inside the reconnection window
    Given monitor is in state Tripped
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.05 | 60 | Normal |
      | 1.06 | 60 | Tripped |
      | 0.917 | 60 | Normal |
      | 0.907 | 60 | Tripped |
      | 1.0 | 60.1 | Normal |
      | 1.0 | 60.2 | Tripped |
      | 1.0 | 59.5 | Normal |
      | 1.0 | 59.4 | Tripped |

  @verifies REQ-GTI-SW-001
  Scenario: the first sample after power-up is classified
    When grid receives Set with v = 1.0, f = 60
    Then monitor is in state Normal
    And supervisor.gridOk == 1

  @verifies REQ-GTI-SYS-010
  Scenario: a dead grid at power-up is not connected to
    When grid receives Set with v = 0, f = 60
    And supervisor receives Start
    And 301 s elapses
    Then supervisor is in state Standby
    And relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0
    When grid receives Set with v = 1.0, f = 60
    And supervisor receives Start
    Then supervisor is in state WaitGrid

  @verifies REQ-GTI-SYS-039,REQ-GTI-HW-008
  Scenario Outline: the hardware trip re-arms exactly at the edge of its range
    Given hwTrip is in state Latched
    When grid receives Set with v = <v>, f = 60
    Then hwTrip is in state <state>

    Examples:
      | v | state |
      | 1.2 | Armed |
      | 1.21 | Latched |
      | 0.45 | Armed |
      | 0.44 | Latched |

  @verifies REQ-GTI-SYS-038,REQ-GTI-SYS-039,REQ-GTI-HW-009
  Scenario: the hardware trip keeps relay B and the gates inhibited when the software path is blind
    Given fault FM-GTI-009 is injected
    When grid receives Set with v = 0, f = 60
    And supervisor receives Start
    And 301 s elapses
    Then relayA.closed == 1
    And relayB.closed == 0
    And relayB.inhibit == 1
    When 200 ms elapses
    Then powerStage.pwm == 0
    And powerStage.inhibit == 1
    When fault FM-GTI-009 is cleared
    And grid receives Set with v = 1.0, f = 60
    Then relayB.inhibit == 0
    And powerStage.inhibit == 0

  @verifies REQ-GTI-SYS-039,REQ-GTI-HW-008
  Scenario Outline: the hardware trip acts on gross voltage loss only
    When grid receives Set with v = <v>, f = 60
    Then hwTrip is in state <state>

    Examples:
      | v | state |
      | 1.2 | Armed |
      | 1.21 | Latched |
      | 0.45 | Armed |
      | 0.44 | Latched |

  @verifies REQ-GTI-SYS-010,REQ-GTI-SW-001
  Scenario: the monitor returns to normal when the grid comes back inside the window
    When grid receives Set with v = 1.0, f = 62.1
    And grid receives Set with v = 1.0, f = 60
    Then monitor is in state Normal
    When 260 ms elapses
    Then supervisor is in state Standby

  @verifies REQ-GTI-SW-008
  Scenario: a grid event during a perturbation measurement discards it instead of tripping
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When 1050 ms elapses
    Then perturb is in state Settling
    When grid receives Set with v = 1.02, f = 60
    Then perturb is in state Idle
    When 2 s elapses
    Then supervisor is in state Running

  @verifies REQ-GTI-SYS-007,REQ-GTI-SYS-010,REQ-GTI-SYS-038
  Scenario: the chain reconnects after a grid loss and clears a second one
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    And 160 ms elapses
    And grid receives Set with v = 1.0, f = 60
    And 301 s elapses
    And 6 s elapses
    Then supervisor is in state Running
    And relayA.closed == 1 and relayB.closed == 1
    When grid receives Set with v = 0, f = 60
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0

  @invariant @verifies REQ-GTI-SYS-007,REQ-GTI-SYS-010,REQ-GTI-SYS-038
  Scenario: a second grid loss after a full reconnection cycle is also cleared
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    And 160 ms elapses
    And grid receives Set with v = 1.0, f = 60
    And 301 s elapses
    And 6 s elapses
    And grid receives Set with v = 0, f = 60
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0

  @invariant @verifies REQ-GTI-SYS-010,REQ-GTI-SYS-038
  Scenario: a dead grid at power-up is never energised
    When grid receives Set with v = 0, f = 60
    And supervisor receives Start
    And 310 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0

  @invariant @verifies REQ-GTI-SYS-038
  Scenario: after a grid loss the grid is never energised for longer than 2 s
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    And supervisor.gridOk is 1
    And monitor is in state Normal
    When grid receives Set with v = 0, f = 60
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0
```
