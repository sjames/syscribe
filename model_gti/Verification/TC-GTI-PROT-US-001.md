---
type: TestCase
id: TC-GTI-PROT-US-001
name: Protection chain behaviour, US profile
status: active
testLevel: L4
subject: Behavior::Chain::ProtectionChain
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
tags: [executable, region-us, reg-unverified]
appliesWhen: Features::Region::US
---

Executable scenarios of the US protection chain. Limits are the unverified defaults of the regulatory matrix.

```gherkin
Feature: Grid connection and protection chain (US)

  Scenario: the inverter connects after the reconnect delay and ramps up
    When supervisor receives Start
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
    When 310 s elapses
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
    When grid receives Set with v = 1.0, f = 62.5
    And 159 ms elapses
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
    When grid receives Set with v = 1.0, f = 56.0
    And 160 ms elapses
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
    When grid receives Set with v = 1.0, f = 62.5
    And 80 ms elapses
    And grid receives Set with v = 1.0, f = 60
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
    When grid receives Set with v = 0.5, f = 60
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
    When grid receives Set with v = 0, f = 60
    Then relayB.closed == 0
    And powerStage.pwm == 0
    When 160 ms elapses
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
    When grid receives Set with v = 0, f = 60
    And 160 ms elapses
    And grid receives Set with v = 1.0, f = 60
    Then supervisor is in state WaitGrid
    When 299 s elapses
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
    When grid receives Set with v = 1.0, f = 62.5
    And 160 ms elapses
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
    When grid receives Set with v = 0, f = 60
    And hwTrip receives Reset
    And grid receives Set with v = 1.0, f = 60
    And 1 s elapses
    Then hwTrip is in state Armed

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

  Scenario Outline: the monitor recovers from Tripped exactly at the window edge
    Given monitor is in state Tripped
    When grid receives Set with v = <v>, f = <f>
    Then monitor is in state <state>

    Examples:
      | v | f | state |
      | 1.2 | 60 | Normal |
      | 1.21 | 60 | Tripped |
      | 0.45 | 60 | Normal |
      | 0.44 | 60 | Tripped |
      | 1.0 | 62 | Normal |
      | 1.0 | 62.1 | Tripped |
      | 1.0 | 56.5 | Normal |
      | 1.0 | 56.4 | Tripped |

  Scenario Outline: the hardware trip acts on gross voltage loss only
    When grid receives Set with v = <v>, f = 60
    Then hwTrip is in state <state>

    Examples:
      | v | state |
      | 1.2 | Armed |
      | 1.21 | Latched |
      | 0.45 | Armed |
      | 0.44 | Latched |

  Scenario: the monitor returns to normal when the grid comes back inside the window
    When grid receives Set with v = 1.0, f = 62.1
    And grid receives Set with v = 1.0, f = 60
    Then monitor is in state Normal
    When 260 ms elapses
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
    When grid receives Set with v = 0, f = 60
    And 2 s elapses
    Then powerStage.pwm == 0 or relayA.closed == 0 or relayB.closed == 0
```
