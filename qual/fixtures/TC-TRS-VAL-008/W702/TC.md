---
id: TC-TST-W702-001
type: TestCase
name: "L3 integration test for ASIL-D requirement (insufficient for W702)"
status: active
testLevel: L3
verifies:
  - REQ-TST-W702-001
---

An active TestCase at `testLevel: L3` that verifies the ASIL D requirement. Because no active L5 (HIL) TestCase exists, W702 is triggered.

```gherkin
Feature: W702 trigger fixture

  Scenario: Integration test at L3 for ASIL-D requirement
    Given the ASIL-D requirement is in scope
    When the integration test is executed at L3
    Then the requirement is covered at integration level only
```
