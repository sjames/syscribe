---
id: TC-TRS-TRACE-007
type: TestCase
testLevel: L3
status: draft
name: "Verify that E315 is emitted for cross-domain supertype: or typedBy: links."
verifies:
  - REQ-TRS-TRACE-007
---

Verify that E315 is emitted for cross-domain supertype: or typedBy: links.

```gherkin
Feature: HW/SW architecture independence

  Scenario: Software PartDef with hardware supertype: produces E315
    Given a PartDef HwBase with domain: hardware
    And a PartDef SwChild with domain: software and supertype: HwBase
    When the tool is invoked
    Then an E315 finding is emitted for SwChild

  Scenario: Hardware PartDef with software supertype: produces E315
    Given a PartDef SwBase with domain: software
    And a PartDef HwChild with domain: hardware and supertype: SwBase
    When the tool is invoked
    Then an E315 finding is emitted for HwChild

  Scenario: Cross-domain Allocation does not produce E315
    Given a software Part and a hardware Part
    And an Allocation element linking the two
    When the tool is invoked
    Then no E315 finding is emitted for the Allocation
```
