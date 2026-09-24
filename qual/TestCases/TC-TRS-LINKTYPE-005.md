---
id: TC-TRS-LINKTYPE-005
type: TestCase
testLevel: L3
status: draft
name: "Verify acyclic link types reject cycles and self-links with E636, and non-acyclic types are not cycle-checked."
verifies:
  - REQ-TRS-LINKTYPE-005
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-005)

  Scenario: a two-element cycle in an acyclic type raises E636
  Scenario: a self-link in an acyclic type raises E636
  Scenario: a cycle in a non-acyclic type raises nothing
  Scenario: an acyclic chain raises nothing
```
