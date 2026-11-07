---
type: TestCase
id: TC-TRS-AMEND-003
name: "W325: scope changes need an owner reference"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/amendments.rs
verifies:
  - REQ-TRS-AMEND-003
tags:
  - adr
  - amendments
---

```gherkin
Feature: Owner references

  Scenario: owner references
    Then a scope-change with no ref, an open question or a Requirement ref is W325
    And an answered question, a ReviewRecord or a person's commit is clean
    And a commit with a Syscribe-Actor trailer is W325, and a clarification needs no ref
```
