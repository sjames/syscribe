---
id: TC-TRS-TRACE-008
type: TestCase
testLevel: L3
status: draft
name: "Verify that E314 is emitted for a deployment package with no hardware Allocation."
verifies:
  - REQ-TRS-TRACE-008
---

Verify that E314 is emitted for a deployment package with no hardware Allocation.

```gherkin
Feature: Deployment package must have hardware Allocation

  Scenario: Deployment package with no hardware Allocation produces E314
    Given a PartDef with isDeploymentPackage: true
    And no Allocation element in the model linking it to a hardware element
    When the tool is invoked
    Then an E314 finding is emitted for that PartDef

  Scenario: Deployment package with a hardware Allocation does not produce E314
    Given a PartDef with isDeploymentPackage: true and domain: software
    And an Allocation element linking it to a PartDef with domain: hardware
    When the tool is invoked
    Then no E314 finding is emitted
```
