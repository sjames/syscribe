---
type: TestCase
id: TC-TRS-IMPACT-001
name: "change-impact reports validity and projection changes on a candidate copy"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/change_impact.rs
verifies:
  - REQ-TRS-IMPACT-001
tags:
  - cli
---

```gherkin
Feature: change impact

  Scenario: validity and projection
    Then a new requires makes the violating Configuration invalid, a changed appliesWhen moves an element between projections, and a parameter range edit breaks the narrowing that relied on it

  Scenario: read-only and honest paths
    Then nothing is written and findings are reported under the real model root

  Scenario: surfaces and diff locations
    Then CLI, MCP and LSP return one payload and diff lists each differing element with file and line
```
