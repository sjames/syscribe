---
id: TC-TRS-PROJ-003
type: TestCase
testLevel: L3
status: draft
name: "Verify escaping-reference detection: structural E226 (error), traceability W019 (warning)."
verifies:
  - REQ-TRS-PROJ-003
---

```gherkin
Feature: Escaping references per configuration
  Scenario: structural escape is an error
    Given an active Part typedBy a PartDef inactive in the configuration
    When validating with --config that configuration
    Then E226 is reported (and not a generic dangling-reference error)
  Scenario: traceability escape is a warning
    Given an active TestCase verifying a Requirement inactive in the configuration
    Then W019 is reported (and not E102)
  Scenario: references between active elements do not escape
    Given an always-active Part typedBy an always-active PartDef
    Then no escape finding is produced
```
