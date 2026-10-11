---
type: TestCase
id: TC-GTI-PROT-IN-001
name: Protection chain behaviour, India profile
status: active
testLevel: L4
subject: Behavior::Chain::ProtectionChainIn
verifies:
  - REQ-GTI-SYS-007
  - REQ-GTI-SYS-009
  - REQ-GTI-SYS-010
  - REQ-GTI-SYS-030
  - REQ-GTI-SYS-037
  - REQ-GTI-SYS-038
  - REQ-GTI-SW-001
  - REQ-GTI-SW-002
  - REQ-GTI-HW-002
  - REQ-GTI-HW-008
tags: [executable, region-in, reg-unverified]
appliesWhen: Features::Region::IN
---

Executable scenarios of the India protection chain. Limits and times are unverified.

```gherkin
Feature: Grid connection and protection chain (India)

  Scenario: the inverter connects after the reconnect delay and ramps up
    When supervisor receives Start
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
    When 2 ms elapses
    Then supervisor is in state Ramping
    And powerStage is in state On
    And powerStage.pwm == 1
    When 4 s elapses
    Then supervisor is in state Ramping
    When 1 s elapses
    Then supervisor is in state Running

  Scenario: insulation below the limit prevents connection until it recovers
    Given supervisor.rIso is 20
    When supervisor receives Start
    Then supervisor is in state InsulationFault
    When 70 s elapses
    Then relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0
    When supervisor.rIso becomes 4000
    And supervisor receives Start
    Then supervisor is in state WaitGrid

  Scenario Outline: the insulation limit is 34 kOhm from standby
    Given supervisor.rIso is <r>
    When supervisor receives Start
    Then supervisor is in state <state>

    Examples:
      | r | state |
      | 33 | InsulationFault |
      | 34 | WaitGrid |
      | 35 | WaitGrid |

  Scenario Outline: the insulation limit is 34 kOhm after a fault
    Given supervisor is in state InsulationFault
    And supervisor.rIso is <r>
    When supervisor receives Start
    Then supervisor is in state <state>

    Examples:
      | r | state |
      | 33 | InsulationFault |
      | 34 | WaitGrid |
      | 35 | WaitGrid |

  Scenario: a persistent high-frequency excursion trips after the clearing time
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 1.0, f = 51.0
    And 199 ms elapses
    Then supervisor is in state Running
    When 1 ms elapses
    Then supervisor is in state Tripped
    And relayA.closed == 0 and relayB.closed == 0
    And powerStage.pwm == 0

  Scenario: a persistent low-frequency excursion also trips
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 1.0, f = 47.0
    And 200 ms elapses
    Then supervisor is in state Tripped
    And powerStage.pwm == 0

  Scenario: an excursion shorter than the clearing time does not trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 1.0, f = 51.0
    And 100 ms elapses
    And grid receives Set with v = 1.0, f = 50
    And 1 s elapses
    Then supervisor is in state Running
    And powerStage.pwm == 1

  Scenario: voltage inside the window does not trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 0.9, f = 50
    And 1 s elapses
    Then supervisor is in state Running

  Scenario: grid loss disconnects at once through the hardware trip
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 0, f = 50
    Then relayB.closed == 0
    And powerStage.pwm == 0
    When 200 ms elapses
    Then supervisor is in state Tripped
    And relayA.closed == 0

  Scenario: the inverter reconnects only after the grid is back and the delay has passed
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 0, f = 50
    And 200 ms elapses
    And grid receives Set with v = 1.0, f = 50
    Then supervisor is in state WaitGrid
    When 59 s elapses
    Then relayA.closed == 0
    When 1 s elapses
    Then relayA.closed == 1

  Scenario Outline: grid loss in any connection phase aborts the connection
    Given supervisor is in state <phase>
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 1.0, f = 51.0
    And 200 ms elapses
    Then supervisor is in state Tripped
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

  Scenario: the hardware trip latches until it is reset
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 0, f = 50
    And hwTrip receives Reset
    And grid receives Set with v = 1.0, f = 50
    And 1 s elapses
    Then hwTrip is in state Armed

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

  Scenario Outline: the monitor recovers from Tripped exactly at the window edge
    Given monitor is in state Tripped
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.1 | 50 | Normal |
      | 1.11 | 50 | Tripped |
      | 0.85 | 50 | Normal |
      | 0.84 | 50 | Tripped |
      | 1.0 | 50.5 | Normal |
      | 1.0 | 50.6 | Tripped |
      | 1.0 | 47.5 | Normal |
      | 1.0 | 47.4 | Tripped |

  Scenario Outline: the hardware trip acts on gross voltage loss only
    When grid receives Set with v = <v>, f = 50
    Then hwTrip is in state <state>

    Examples:
      | v | state |
      | 1.2 | Armed |
      | 1.21 | Latched |
      | 0.45 | Armed |
      | 0.44 | Latched |

  Scenario: the monitor returns to normal when the grid comes back inside the window
    When grid receives Set with v = 1.0, f = 50.6
    And grid receives Set with v = 1.0, f = 50
    Then monitor is in state Normal
    When 300 ms elapses
    Then supervisor is in state Standby

  @invariant
  Scenario: after a grid loss the grid is never energised for longer than 2 s
    Given supervisor is in state Running
    And relayA is in state Closed
    And relayA.closed is 1
    And relayB is in state Closed
    And relayB.closed is 1
    And powerStage is in state On
    And powerStage.pwm is 1
    When grid receives Set with v = 0, f = 50
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0
```
