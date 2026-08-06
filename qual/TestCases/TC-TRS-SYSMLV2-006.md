---
id: TC-TRS-SYSMLV2-006
type: TestCase
testLevel: L3
status: draft
name: "Verify a malformed sysmlSubmodel: value, a .sysml parse failure, and an unmapped construct each degrade gracefully without aborting validate."
verifies:
  - REQ-TRS-SYSMLV2-006
---

```gherkin
Feature: SysMLv2 ingestion failures degrade gracefully
  Scenario: a malformed sysmlSubmodel: value does not abort validation
    Given a package _index.md with sysmlSubmodel set to a non-boolean value
    When the model is validated
    Then a Finding names that file and the rest of the model still validates

  Scenario: a .sysml parse failure does not abort validation
    Given a .sysml file with a syntax error
    When the model is validated
    Then a Finding names that file, it contributes zero elements, and validation completes

  Scenario: an unmapped construct produces no finding at all
    Given a .sysml file containing only constructs outside the mapped element set
    When the model is validated
    Then validation completes with zero errors and zero warnings attributable to that file
```
