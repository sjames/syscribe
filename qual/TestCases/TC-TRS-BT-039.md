---
id: TC-TRS-BT-039
type: TestCase
testLevel: L3
status: active
name: "Verify every binding-time field, code and command is documented and registered"
verifies:
  - REQ-TRS-BT-039
---

```gherkin
Feature: Documentation and catalogue
  Scenario: catalogue
    Given every code added by this feature
    When catalogue_tests and validation_docs_catalogue are run
    Then they pass

  Scenario: example
    Given examples/ staged-configuration worked example
    When validate is run
    Then it is clean
```
