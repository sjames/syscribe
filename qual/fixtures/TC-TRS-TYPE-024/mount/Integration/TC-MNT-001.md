---
id: TC-MNT-001
type: TestCase
name: "Verifies the peer requirement through the mount point"
status: draft
testLevel: L3
verifies: [Integration::Brakes::REQ-BRK-001]
---
```gherkin
Feature: Mounted verification
  Scenario: mounted
    Given the brake system
    Then it brakes
```
