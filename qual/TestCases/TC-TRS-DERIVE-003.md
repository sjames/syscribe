---
id: TC-TRS-DERIVE-003
type: TestCase
name: "Null coalesce and custom_fields arithmetic evaluate correctly"
status: approved
testLevel: L1
verifies: [REQ-TRS-DERIVE-003]
---

An element with a `derive:` formula using null coalesce (`??`) and arithmetic on `self.custom_fields` values shall produce the correct numeric result.

```gherkin
Feature: Null coalesce and custom_fields arithmetic evaluate correctly

  Scenario: Null coalesce and custom_fields arithmetic evaluate correctly
Given a PartDef element with custom_fields: { budget: 100, used: 60 }
  And a derive: block: { headroom: "self.custom_fields.budget - self.custom_fields.used", ratio: "self.custom_fields.used / (self.custom_fields.budget ?? 1)" }
When I run: syscribe -m <fixture> query show Sys::Task
Then the output contains "headroom" with value 40
And the output contains "ratio" with value 0.6
And validation reports no errors
```
