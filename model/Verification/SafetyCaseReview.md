---
type: TestCase
id: TC-UAV-SAFE-SYS-001
name: "Safety case review confirms UAV shall not cause injury across all flight phases"
status: active
testLevel: L2
verifies:
  - REQ-UAV-SAFE-000
tags:
  - safety
  - review
  - safety-case
---

Structured safety case review conducted by the Safety Review Board. The review checks that every derived safety requirement (REQ-UAV-FC-001, REQ-UAV-SAFE-001) has been verified, that the fault tree is complete, and that residual risk is acceptable.

```gherkin
Feature: Safety case completeness review

  Background:
    Given all derived safety requirements have status: verified
    And the fault tree analysis document is available

  Scenario: All derived safety requirements are verified
    When the safety review board examines the requirement coverage matrix
    Then REQ-UAV-FC-001 shall have at least one active TestCase with status: verified
    And REQ-UAV-SAFE-001 shall have at least one active TestCase with status: verified

  Scenario: Residual risk is within acceptable bounds
    When the fault tree analysis is reviewed
    Then no unmitigated hazard with severity class IV or above shall remain open
```
