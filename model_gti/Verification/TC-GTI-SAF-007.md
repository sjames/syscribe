---
type: TestCase
id: TC-GTI-SAF-007
name: "Hardware overcurrent trip"
status: active
testLevel: L2
verifies:
  - REQ-GTI-SYS-037
---

```gherkin
Feature: Bench test of the comparator path

  Scenario: software disabled
    Given the firmware is halted
    When the AC current is driven to twice the rated peak
    Then the gate drivers turn off within 10 µs
```
