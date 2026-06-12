---
id: TC-TRS-CFLD-003
type: TestCase
testLevel: L3
status: draft
name: "Verify show renders a custom-fields section when present and omits it when absent."
verifies:
  - REQ-TRS-CFLD-003
---

```gherkin
Feature: custom fields in show
  Scenario: rendering
    Given an element with custom_fields and one without
    When show is invoked on each
    Then the first prints a Custom Fields section with scalar and list values
    And the second prints no custom-fields section
```
