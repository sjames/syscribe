---
type: TestCase
id: TC-TRS-AMEND-004
name: "Amendment surfaces: set, show, export-html, suspect, template and help"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/amendments.rs
verifies:
  - REQ-TRS-AMEND-004
tags:
  - adr
  - amendments
---

```gherkin
Feature: Amendment surfaces

  Scenario: set amendment.add
    Then it fills date and by, and refuses a bad kind, an unresolved ref, no author and a non-host type

  Scenario: show and export-html
    Then both render the amendment history table

  Scenario: suspect links
    Then an amendment entry alone leaves a baselined breakdownAdr link clean, and a body edit makes it W090

  Scenario: template and help
    Then template and help for ADR and Requirement show amendments, and help --search amend finds it
```
