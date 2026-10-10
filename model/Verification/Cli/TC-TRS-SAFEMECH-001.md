---
type: TestCase
id: TC-TRS-SAFEMECH-001
name: "safety mechanisms validate, are checked against FTTI and are listed"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/safety_mechanism.rs
verifies:
  - REQ-TRS-SAFEMECH-001
tags:
  - safety
---

```gherkin
Feature: safety mechanisms

  Scenario: validity
    Then a valid mechanism is clean and bad ids, statuses, coverages and covers are E896/E897

  Scenario: reaction time
    Then a reaction time over the covered goal's FTTI is W894

  Scenario: listing
    Then mechanisms lists coverage and --uncovered lists FMEA rows nothing covers
```
