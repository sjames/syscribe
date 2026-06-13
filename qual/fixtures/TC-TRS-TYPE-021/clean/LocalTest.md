---
id: TC-INT-AA-001
type: TestCase
testLevel: L3
status: draft
name: Verifies a peer requirement across repos
verifies:
  - REQ-PEER-AA-001
---
Cross-repo verification.
```gherkin
Feature: cross-repo
  Scenario: peer requirement is verified
    Given the composition
    When the test runs
    Then the peer requirement is satisfied
```
