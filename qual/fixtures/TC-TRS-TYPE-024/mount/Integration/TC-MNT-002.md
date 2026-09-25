---
id: TC-MNT-002
type: TestCase
name: "Verifies the peer requirement by peer-native qname and by stable id"
status: draft
testLevel: L3
verifies: [BrakeSystem::REQ-BRK-001, REQ-BRK-001]
---
```gherkin
Feature: Native verification
  Scenario: native
    Given the brake system
    Then it brakes
```
