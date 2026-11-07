---
type: TestCase
id: TC-TRS-AMEND-001
name: "amendments: shape, ref resolution and placement"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/amendments.rs
verifies:
  - REQ-TRS-AMEND-001
tags:
  - adr
  - amendments
---

```gherkin
Feature: Amendment records

  Scenario: well-formed list
    Then an ADR and a Requirement with well-formed amendments raise no E330 and no W047

  Scenario: malformed entries
    Then a non-list, a missing reason, a bad kind, a bad date and an unresolved ref are each E330

  Scenario: ref resolution
    Then a question, a ReviewRecord, a Requirement and a commit of the repository resolve

  Scenario: other types
    Then amendments on a PartDef is W047
```
