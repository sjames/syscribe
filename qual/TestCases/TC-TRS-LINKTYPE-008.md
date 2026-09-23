---
id: TC-TRS-LINKTYPE-008
type: TestCase
testLevel: L3
status: draft
name: "Verify link-types lists the declared vocabulary and rules in text and JSON, omits invalid entries, and hints when none are declared."
verifies:
  - REQ-TRS-LINKTYPE-008
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-008)

  Scenario: text output lists each type and its rules
  Scenario: json output lists each type and its rules
  Scenario: an invalid entry is not listed as usable
  Scenario: an unconfigured model prints a hint and exits zero
```
