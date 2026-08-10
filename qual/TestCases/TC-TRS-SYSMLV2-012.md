---
id: TC-TRS-SYSMLV2-012
type: TestCase
testLevel: L3
status: draft
name: "Verify a named connection usage's own trailing doc /* ... */ body lifts into the synthesized Connection element's doc field, with no regression for a connection usage with no trailing body."
verifies:
  - REQ-TRS-SYSMLV2-012
---

```gherkin
Feature: a named connection usage's own trailing doc body lifts
  Scenario: a connection usage's trailing doc body lifts
    Given a SysMLv2 named connection usage with a trailing { doc /* ... */ } body
    When the model is exported
    Then the synthesized Connection element's doc body matches the comment text

  Scenario: a connection usage with no trailing body is unaffected
    Given a SysMLv2 named connection usage with no trailing body
    When the model is exported
    Then the synthesized Connection element's doc body is empty
```
