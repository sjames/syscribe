---
id: TC-TRS-IMPL-001
type: TestCase
testLevel: L3
status: draft
name: "Verify implementedBy path-exists rule W023: missing path, opt-in, draft suppression, gating."
verifies:
  - REQ-TRS-IMPL-001
---

```gherkin
Feature: implementedBy path-exists (W023)
  Scenario: missing implementation path is W023
    Given a non-draft PartDef whose implementedBy path is absent on disk
    When the tool validates the model
    Then a W023 finding names that element
  Scenario: present path, opt-in, and draft are not flagged
    Given a PartDef whose path exists, one with no implementedBy, and a draft one with a missing path
    Then none of them produce W023
  Scenario: W023 is gateable
    When validating with --deny W023
    Then the tool exits 2
```
