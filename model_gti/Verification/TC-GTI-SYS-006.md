---
type: TestCase
id: TC-GTI-SYS-006
name: "Partial shading global scan"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-023
appliesWhen: Features::Mppt::GlobalScan
---

```gherkin
Feature: Two-peak P-V curve

  Scenario: global scan finds the higher peak
    Given a PV simulator with a local peak at 300 V and a global peak at 700 V
    When a scan period elapses
    Then the tracker moves to the 700 V peak within one scan duration
    And the gain over staying at the local peak is reported
```
