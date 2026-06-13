---
id: TC-TRS-DERIVE-001
type: TestCase
name: "Basic derive block evaluates and appears in query show output"
status: approved
testLevel: L1
verifies: [REQ-TRS-DERIVE-001]
---

A `PartDef` element with a `derive:` block containing simple arithmetic and a self-referencing later field shall evaluate both fields and include them under **Derived Fields** in `query show` output.

```gherkin
Feature: Basic derive block evaluates and appears in query show output

  Scenario: Basic derive block evaluates and appears in query show output
Given a model containing a PartDef with derive: block
  And the first derived field is a constant formula
  And the second derived field references the first via self.<field>
When I run: syscribe -m <fixture> query show Sys::Widget
Then the output contains "## Derived Fields"
And the output contains the first field name and its value
And the output contains the second field name and its value
And validation reports no errors
```
