---
id: TC-TRS-LINKTYPE-003
type: TestCase
testLevel: L3
status: draft
name: "Verify sourceTypes and targetTypes constraints raise E633 and E634."
verifies:
  - REQ-TRS-LINKTYPE-003
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-003)

  Scenario: permitted source and target validate cleanly
  Scenario: a forbidden source type raises E633
  Scenario: a forbidden target type raises E634
```
