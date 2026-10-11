---
type: TestCase
id: TC-TRS-SYSMUT-001
name: "Mutation of a composite's parts is judged by the composite scenarios"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/syscomp_mut.rs
verifies:
  - REQ-TRS-SYSMUT-001
tags:
  - mutation
---

```gherkin
Feature: Composite mutation

  Scenario: a weak scenario set lets a mutant of one part survive
    Then the survivor is listed under its part and the score of that part is below 100

  Scenario: a thorough scenario set kills it
    Then adding a scenario that distinguishes the boundary raises the part's score to 100

  Scenario: gates and filters
    Then --part restricts the analysis, --min-score fails a part below the threshold and names it, and scenarios that fail on the unmutated composite stop the analysis
```
