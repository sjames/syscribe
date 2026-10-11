---
type: TestCase
id: TC-GTI-MCU-003
name: "Resource margin"
status: active
testLevel: L3
verifies:
  - REQ-GTI-SYS-072
---

```gherkin
Feature: Resources

  Scenario: CPU and memory
    Given the release build on the target
    When the fastest control task and memory use are measured
    Then CPU use is at most 70 % and RAM and flash use at most 80 %
```
