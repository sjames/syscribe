---
id: TC-INT-AA-002
type: TestCase
testLevel: L3
status: draft
name: Verifies a requirement that exists in no repo
verifies:
  - REQ-GHOST-ZZ-999
---
```gherkin
Feature: dangling
  Scenario: nowhere
    Given x
    When y
    Then z
```
