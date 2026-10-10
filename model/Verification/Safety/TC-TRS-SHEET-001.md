---
type: TestCase
id: TC-TRS-SHEET-001
name: "Sheet-level responsibility, appliesWhen and tags are inherited by exploded rows"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/sheet_field_inheritance.rs
verifies:
  - REQ-TRS-SHEET-001
tags:
  - safety
---

```gherkin
Feature: Sheet-level field inheritance

  Scenario: TARA rows inherit
    Given a TARASheet with responsibility, appliesWhen and tags and two goal rows, one overriding responsibility
    Then the first row carries the sheet values and the second its own responsibility

  Scenario: FMEA entries inherit
    Given an FMEASheet with responsibility, appliesWhen and tags and one entry
    Then the entry carries them

  Scenario: W038
    Given another element declares responsibility so W038 is active
    Then a sheet with sheet-level responsibility yields no W038 for its rows
```
