---
id: TC-TRS-CFLD-001
type: TestCase
testLevel: L3
status: draft
title: "Verify custom_fields shape validation: scalars/lists clean, nested map raises W041."
verifies:
  - REQ-TRS-CFLD-001
---

```gherkin
Feature: custom_fields shape validation
  Scenario: valid shapes are clean
    Given an element whose custom_fields are scalars and lists of scalars
    When validate runs
    Then no W041 is raised

  Scenario: a nested map is flagged
    Given an element whose custom_fields value is a nested map
    When validate runs
    Then W041 is raised naming the offending key
```
