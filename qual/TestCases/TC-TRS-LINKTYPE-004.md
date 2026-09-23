---
id: TC-TRS-LINKTYPE-004
type: TestCase
testLevel: L3
status: draft
name: "Verify cardinality bounds: E635 above the upper bound, W631 below the lower bound for non-draft in-scope elements."
verifies:
  - REQ-TRS-LINKTYPE-004
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-004)

  Scenario: within bounds validates cleanly
  Scenario: more targets than the upper bound raises E635
  Scenario: a non-draft in-scope element under the lower bound raises W631
  Scenario: a draft element under the lower bound raises nothing
```
