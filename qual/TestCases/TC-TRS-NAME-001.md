---
id: TC-TRS-NAME-001
type: TestCase
testLevel: L3
status: draft
title: "Verify SysMLv2 basic-name validation: W042 on non-basic names, exempting stable ids; hyphenated appliesWhen still E209."
verifies:
  - REQ-TRS-NAME-001
---

```gherkin
Feature: basic-name validation (W042)
  Scenario: non-basic names are flagged, basic names and stable ids are not
    Given a model with a hyphenated FeatureDef name, an underscore name, and a stable-id-named requirement
    When validate runs
    Then W042 is raised naming the hyphenated segment
    And the underscore name is not flagged
    And the stable-id-named element is not flagged
    And a hyphenated appliesWhen reference still raises E209
```
