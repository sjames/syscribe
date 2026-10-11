---
type: TestCase
id: TC-GTI-MCU-001
name: "STM32 controller inspection"
status: active
testLevel: L2
verifies:
  - REQ-GTI-STK-007
  - REQ-GTI-SYS-070
---

```gherkin
Feature: Bill of materials

  Scenario: controller is an STM32
    Given the released bill of materials
    When the control computer is inspected
    Then it is one STM32 part and matches ADR-GTI-MCU-001
```
