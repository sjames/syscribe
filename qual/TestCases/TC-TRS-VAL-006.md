---
id: TC-TRS-VAL-006
type: TestCase
testLevel: L3
status: draft
name: "Verify that E-code findings are marked Error and W-code findings are marked Warning."
verifies:
  - REQ-TRS-VAL-007
---

Verify that E-code findings are marked Error and W-code findings are marked Warning.

```gherkin
Feature: Finding severity classification

  Scenario: E-code findings carry Error severity
    Given a model that triggers at least one E-code rule
    When the tool output is parsed
    Then every finding with an E-nnn code is labelled as Error severity
    And no E-nnn code finding is labelled as Warning

  Scenario: W-code findings carry Warning severity
    Given a model that triggers at least one W-code rule
    When the tool output is parsed
    Then every finding with a W-nnn code is labelled as Warning severity
    And no W-nnn code finding is labelled as Error
```
