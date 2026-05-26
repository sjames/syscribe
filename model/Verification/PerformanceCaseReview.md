---
type: TestCase
id: TC-UAV-PERF-001
title: "Performance requirements review confirms mission objectives are fully covered"
status: active
testLevel: L2
verifies:
  - REQ-UAV-PERF-000
tags:
  - performance
  - review
---

Review confirming that all mission performance requirements (endurance, navigation, data link) derived from the mission performance goal have been verified.

```gherkin
Feature: Mission performance requirements coverage

  Background:
    Given the requirements traceability matrix is current

  Scenario: All mission performance leaf requirements are verified
    When the traceability matrix is reviewed
    Then REQ-UAV-ENDUR-001 shall have at least one active verified TestCase
    And REQ-UAV-NAV-001 shall have at least one active verified TestCase
    And REQ-UAV-COMM-001 shall have at least one active verified TestCase
```
