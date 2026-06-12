---
id: TC-TRS-LIB-001
type: TestCase
testLevel: L3
status: draft
name: "Verify built-in type recognition: ScalarValues/Base members resolve with no W404/W043; unknown members raise W043; import-only packages stay lenient."
verifies:
  - REQ-TRS-LIB-001
---

```gherkin
Feature: recognise auto-imported standard-library types (ScalarValues, Base)
  Scenario: recognised members resolve cleanly
    Given a model using ScalarValues::{Real,Integer,Boolean,String,Natural} and Base::DataValue
    When validate runs
    Then no W404 and no W043 are raised for them

  Scenario: an import-only package reference stays lenient
    Given an operation parameter typed by SI::kg
    When validate runs
    Then no W043 is raised for the SI reference

  Scenario: an unknown member of a known built-in package is flagged
    Given references to ScalarValues::Flota / ScalarValues::Flta / Base::Booleen / Base::Nope
    When validate runs
    Then W043 is raised for each, naming the bad member and listing the package's known members

  Scenario: the check covers every type-reference context
    Given the typos appear in supertype, feature typedBy, operation returnType, and a parameter type
    When validate runs
    Then W043 is raised in each context
```
