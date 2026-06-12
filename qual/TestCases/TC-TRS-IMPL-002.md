---
id: TC-TRS-IMPL-002
type: TestCase
testLevel: L3
status: draft
name: "Verify implementedBy discoverability: links, refs, spec fields."
verifies:
  - REQ-TRS-IMPL-002
---

```gherkin
Feature: implementedBy discoverability
  Scenario: links shows the implementation path
    When listing links for a PartDef with implementedBy
    Then the implementation path is shown
  Scenario: refs on a module path reports the owning element
    When querying refs for a module path
    Then the architecture element that declares it is reported
  Scenario: spec fields documents implementedBy
    When printing spec fields
    Then implementedBy is listed
```
