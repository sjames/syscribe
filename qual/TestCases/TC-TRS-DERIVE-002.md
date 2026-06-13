---
id: TC-TRS-DERIVE-002
type: TestCase
name: "Aggregate operators sum and count work over children"
status: approved
testLevel: L1
verifies: [REQ-TRS-DERIVE-002]
---

A parent `PartDef` with a `derive:` block using `sum(children.custom_fields.wcet)` and `count(children)` shall aggregate values from its direct children.

```gherkin
Given a model with a parent PartDef with two child PartDef elements
  And each child has a custom_fields.wcet value
  And the parent has: derive: { totalWcet: "sum(children.custom_fields.wcet)", childCount: "count(children)" }
When I run: syscribe -m <fixture> query show Sys::Parent
Then the output contains "totalWcet"
And the totalWcet value equals the sum of child wcet values
And the childCount value equals 2
```
