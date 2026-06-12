---
id: TC-TRS-NAME-001
type: TestCase
testLevel: L3
status: draft
name: "Verify SysMLv2 basic-name validation: W042 on non-basic names, exempting stable ids; hyphenated appliesWhen still E209."
verifies:
  - REQ-TRS-NAME-001
---

```gherkin
Feature: basic-name validation (W042)
  Scenario: non-basic names are flagged, basic names and stable ids are not
    Given a model with a hyphenated FeatureDef name, an underscore name, and a stable-id-named requirement
    When validate runs
    Then W042 is raised naming the hyphenated segment using the term "qualified-name segment"
    And the W042 finding on the FeatureDef includes an E209 consequence hint
    And the underscore name is not flagged
    And the stable-id-named element is not flagged
    And a hyphenated appliesWhen reference still raises E209

  Scenario: id-identified element with id+description filename is exempt from W042
    Given an id-identified element (AssumptionOfUse) whose file stem is id + description (e.g. AOU-001-Desc.md) with an explicit name: label
    When validate runs
    Then W042 is not raised for the id+description file stem
```
