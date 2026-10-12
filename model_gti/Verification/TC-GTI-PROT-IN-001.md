---
type: TestCase
id: TC-GTI-PROT-IN-001
name: "Protection chain behaviour, India profile"
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
  - REQ-GTI-HW-008
  - REQ-GTI-HW-009
tags: [executable, region-in, reg-unverified]
appliesWhen: Features::Region::IN
---

Executable scenarios of the protection chain with the India grid profile. Run with `syscribe simulate TC-GTI-PROT-IN-001 --config CONF-GTI-IN-M3-001`. Limits and times are unverified.

```gherkin
Feature: Grid connection and protection chain (India)

  @verifies REQ-GTI-SYS-010
  Scenario: the inverter connects after the reconnect delay and ramps up
    When grid receives Set with v = 1.0, f = 50
    And supervisor receives Start
    Then supervisor is in state WaitGrid
    When 59 s elapses
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
    When grid receives Set with v = 1.0, f = 50
    And supervisor receives Start
    Then supervisor is in state InsulationFault
    When 70 s elapses
    Then relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0
    When supervisor.rIso becomes 4000
    And supervisor receives Start
    Then supervisor is in state WaitGrid

  @verifies REQ-GTI-SYS-030
  Scenario Outline: the insulation limit is 34 kOhm from standby
    Given supervisor.rIso is <r>
    When grid receives Set with v = 1.0, f = 50
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
    When grid receives Set with v = 1.0, f = 51.0
    And 199 ms elapses
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
    When grid receives Set with v = 1.0, f = 47.0
    And 200 ms elapses
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
    When grid receives Set with v = 1.0, f = 51.0
    And 100 ms elapses
    And grid receives Set with v = 1.0, f = 50
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
    When grid receives Set with v = 0.9, f = 50
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
    When grid receives Set with v = 0, f = 50
    Then relayB.closed == 0
    And powerStage.pwm == 0
    When 200 ms elapses
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
    When grid receives Set with v = 0, f = 50
    And 200 ms elapses
    And grid receives Set with v = 1.0, f = 50
    Then supervisor is in state WaitGrid
    When 59 s elapses
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
    When grid receives Set with v = 1.0, f = 51.0
    And 200 ms elapses
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
    When grid receives Set with v = 0, f = 50
    Then hwTrip is in state Latched
    When grid receives Set with v = 0.3, f = 50
    Then hwTrip is in state Latched
    When grid receives Set with v = 1.0, f = 50
    Then hwTrip is in state Armed
    And relayB.closed == 0

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor classifies samples at the window boundaries
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.1 | 50 | Normal |
      | 1.11 | 50 | Excursion |
      | 0.85 | 50 | Normal |
      | 0.84 | 50 | Excursion |
      | 1.0 | 50.5 | Normal |
      | 1.0 | 50.6 | Excursion |
      | 1.0 | 47.5 | Normal |
      | 1.0 | 47.4 | Excursion |

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor classifies samples at the window boundaries when normal
    Given monitor is in state Normal
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.1 | 50 | Normal |
      | 1.11 | 50 | Excursion |
      | 0.85 | 50 | Normal |
      | 0.84 | 50 | Excursion |
      | 1.0 | 50.5 | Normal |
      | 1.0 | 50.6 | Excursion |
      | 1.0 | 47.5 | Normal |
      | 1.0 | 47.4 | Excursion |

  @verifies REQ-GTI-SYS-007,REQ-GTI-SW-001
  Scenario Outline: the monitor recovers from Excursion exactly at the window edge
    Given monitor is in state Excursion
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.1 | 50 | Normal |
      | 1.11 | 50 | Excursion |
      | 0.85 | 50 | Normal |
      | 0.84 | 50 | Excursion |
      | 1.0 | 50.5 | Normal |
      | 1.0 | 50.6 | Excursion |
      | 1.0 | 47.5 | Normal |
      | 1.0 | 47.4 | Excursion |

  @verifies REQ-GTI-SYS-010,REQ-GTI-SW-001
  Scenario Outline: the monitor reports the grid good again only inside the reconnection window
    Given monitor is in state Tripped
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.1 | 50 | Normal |
      | 1.11 | 50 | Tripped |
      | 0.9 | 50 | Normal |
      | 0.89 | 50 | Tripped |
      | 1.0 | 50.5 | Normal |
      | 1.0 | 50.6 | Tripped |
      | 1.0 | 49.5 | Normal |
      | 1.0 | 49.4 | Tripped |

  @verifies REQ-GTI-SW-001
  Scenario: the first sample after power-up is classified
    When grid receives Set with v = 1.0, f = 50
    Then monitor is in state Normal
    And supervisor.gridOk == 1

  @verifies REQ-GTI-SYS-010
  Scenario: a dead grid at power-up is not connected to
    When grid receives Set with v = 0, f = 50
    And supervisor receives Start
    And 61 s elapses
    Then supervisor is in state Standby
    And relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0
    When grid receives Set with v = 1.0, f = 50
    And supervisor receives Start
    Then supervisor is in state WaitGrid

  @verifies REQ-GTI-SYS-039,REQ-GTI-HW-008
  Scenario Outline: the hardware trip re-arms exactly at the edge of its range
    Given hwTrip is in state Latched
    When grid receives Set with v = <v>, f = 50
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
    When grid receives Set with v = 0, f = 50
    And supervisor receives Start
    And 61 s elapses
    Then relayA.closed == 1
    And relayB.closed == 0
    And relayB.inhibit == 1
    When 200 ms elapses
    Then powerStage.pwm == 0
    And powerStage.inhibit == 1
    When fault FM-GTI-009 is cleared
    And grid receives Set with v = 1.0, f = 50
    Then relayB.inhibit == 0
    And powerStage.inhibit == 0

  @verifies REQ-GTI-SYS-039,REQ-GTI-HW-008
  Scenario Outline: the hardware trip acts on gross voltage loss only
    When grid receives Set with v = <v>, f = 50
    Then hwTrip is in state <state>

    Examples:
      | v | state |
      | 1.2 | Armed |
      | 1.21 | Latched |
      | 0.45 | Armed |
      | 0.44 | Latched |

  @verifies REQ-GTI-SYS-010,REQ-GTI-SW-001
  Scenario: the monitor returns to normal when the grid comes back inside the window
    When grid receives Set with v = 1.0, f = 50.6
    And grid receives Set with v = 1.0, f = 50
    Then monitor is in state Normal
    When 300 ms elapses
    Then supervisor is in state Standby

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
    When grid receives Set with v = 0, f = 50
    And 200 ms elapses
    And grid receives Set with v = 1.0, f = 50
    And 61 s elapses
    And 6 s elapses
    Then supervisor is in state Running
    When grid receives Set with v = 0, f = 50
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0

  @invariant @verifies REQ-GTI-SYS-010,REQ-GTI-SYS-038
  Scenario: a dead grid at power-up is never energised
    When grid receives Set with v = 0, f = 50
    And supervisor receives Start
    And 70 s elapses
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
    When grid receives Set with v = 0, f = 50
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0
```
