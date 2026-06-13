---
id: TC-TRS-DERIVE-005
type: TestCase
name: "Cross-element reference to unknown element emits E502"
status: approved
testLevel: L1
verifies: [REQ-TRS-DERIVE-005]
---

A `derive:` formula that references a nonexistent element via `elements["Qname"]` shall emit error **E502** and produce a null value for that field.

```gherkin
Feature: Cross-element reference to unknown element emits E502

  Scenario: Cross-element reference to unknown element emits E502
Given a PartDef with derive: { refVal: 'elements["NonExistent::Thing"].custom_fields.x' }
When I run: syscribe -m <fixture>
Then the output contains "E502"
And the output contains "NonExistent::Thing"
And validation reports the E502 finding
```
