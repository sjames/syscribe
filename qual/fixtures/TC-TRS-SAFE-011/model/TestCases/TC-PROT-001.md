---
id: TC-PROT-001
type: TestCase
testLevel: L3
status: draft
name: "Verify overcurrent response time"
verifies:
  - REQ-PROT-001
---

```gherkin
Feature: Overcurrent protection

  Scenario: Fast overcurrent response
    Given normal operation
    When overcurrent is detected
    Then protection engages within 1ms
```
