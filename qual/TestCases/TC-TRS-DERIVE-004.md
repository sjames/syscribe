---
id: TC-TRS-DERIVE-004
type: TestCase
name: "Invalid derive formula emits E501 parse error"
status: approved
testLevel: L1
verifies: [REQ-TRS-DERIVE-005]
---

An element whose `derive:` block contains a syntactically invalid formula string shall emit error **E501** during validation.

```gherkin
Feature: Invalid derive formula emits E501 parse error

  Scenario: Invalid derive formula emits E501 parse error
Given a PartDef with derive: { broken: "sum(" }
When I run: syscribe -m <fixture>
Then the output contains "E501"
And the output contains "derive formula parse error"
And the output contains "broken"
```
