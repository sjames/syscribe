---
type: TestCase
id: TC-GTI-MPPT-001
name: MPPT tracker behaviour
status: active
testLevel: L3
subject: Behavior::Machines::MpptTracker
verifies:
  - REQ-GTI-SYS-022
  - REQ-GTI-SYS-023
  - REQ-GTI-SW-006
tags: [executable, mppt]
appliesWhen: Features::Mppt::GlobalScan
---

```gherkin
Feature: Perturb-and-observe tracking with a global scan

  Scenario: the tracker climbs while power rises
    When Sample is injected with p = 1000
    And Sample is injected with p = 1100
    Then vRef == 610
    And dir == 1

  Scenario: the tracker reverses when power falls
    When Sample is injected with p = 1000
    And Sample is injected with p = 1100
    And Sample is injected with p = 1050
    Then vRef == 605
    And dir == -1

  Scenario: equal power keeps the direction
    When Sample is injected with p = 1000
    And Sample is injected with p = 1000
    And Sample is injected with p = 1000
    Then vRef == 615
    And dir == 1

  Scenario: no scan before the scan period
    When 599 s elapses
    Then MpptTracker is in state Tracking

  Scenario: the scan finds the global maximum of a two-peak curve
    When 600 s elapses
    Then MpptTracker is in state Scanning
    And vRef == 200
    When Sample is injected with p = 700
    And Sample is injected with p = 780
    And Sample is injected with p = 800
    And Sample is injected with p = 600
    And Sample is injected with p = 500
    And Sample is injected with p = 450
    And Sample is injected with p = 600
    And Sample is injected with p = 900
    And Sample is injected with p = 1200
    And Sample is injected with p = 1400
    And Sample is injected with p = 1500
    And Sample is injected with p = 1300
    And Sample is injected with p = 900
    And Sample is injected with p = 800
    And Sample is injected with p = 100
    Then MpptTracker is in state Tracking
    And vRef == 700
    And pPrev == 1500

  Scenario: the upper window edge is evaluated
    When 600 s elapses
    When Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 2000
    And Sample is injected with p = 100
    Then vRef == 850

  Scenario: the lower window edge is evaluated
    When 600 s elapses
    When Sample is injected with p = 2000
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    Then vRef == 200

  Scenario: of two equal peaks the first one wins
    When 600 s elapses
    When Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 1500
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 1500
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    Then vRef == 300

  Scenario: a scan in the dark returns to the lower window edge
    When 600 s elapses
    When Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    And Sample is injected with p = 0
    Then vRef == 200

  Scenario: tracking resumes upward after a scan
    When Sample is injected with p = 1000
    And Sample is injected with p = 900
    Then dir == -1
    When 600 s elapses
    When Sample is injected with p = 700
    And Sample is injected with p = 780
    And Sample is injected with p = 800
    And Sample is injected with p = 600
    And Sample is injected with p = 500
    And Sample is injected with p = 450
    And Sample is injected with p = 600
    And Sample is injected with p = 900
    And Sample is injected with p = 1200
    And Sample is injected with p = 1400
    And Sample is injected with p = 1500
    And Sample is injected with p = 1300
    And Sample is injected with p = 900
    And Sample is injected with p = 800
    And Sample is injected with p = 100
    Then dir == 1

  Scenario: a second scan starts from scratch
    When 600 s elapses
    When Sample is injected with p = 700
    And Sample is injected with p = 780
    And Sample is injected with p = 800
    And Sample is injected with p = 600
    And Sample is injected with p = 500
    And Sample is injected with p = 450
    And Sample is injected with p = 600
    And Sample is injected with p = 900
    And Sample is injected with p = 1200
    And Sample is injected with p = 1400
    And Sample is injected with p = 1500
    And Sample is injected with p = 1300
    And Sample is injected with p = 900
    And Sample is injected with p = 800
    And Sample is injected with p = 100
    And 600 s elapses
    When Sample is injected with p = 300
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    And Sample is injected with p = 100
    Then vRef == 200
```
