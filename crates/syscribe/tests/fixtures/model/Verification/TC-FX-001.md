---
type: TestCase
id: TC-FX-001
name: "Fixture test case verifying REQ-FX-001"
status: draft
testLevel: L2
verifies:
  - REQ-FX-001
---

```gherkin
Feature: Fixture verification
  Scenario: the fixture requirement is verified
    Given the fixture model
    Then REQ-FX-001 is covered by this test case
```
