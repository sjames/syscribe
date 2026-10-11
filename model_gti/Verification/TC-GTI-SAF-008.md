---
type: TestCase
id: TC-GTI-SAF-008
name: "Relay weld detection"
status: active
testLevel: L2
verifies:
  - REQ-GTI-SYS-038
  - REQ-GTI-HW-002
---

```gherkin
Feature: Series relays

  Scenario: welded relay prevents connection
    Given one relay is replaced by a welded dummy
    When the inverter attempts to connect
    Then the weld is detected from the auxiliary contact
    And the connection is refused
```
