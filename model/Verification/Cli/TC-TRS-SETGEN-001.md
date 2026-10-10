---
type: TestCase
id: TC-TRS-SETGEN-001
name: "generic set edits scalar and list fields and refuses invalid edits"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/set_generic.rs
verifies:
  - REQ-TRS-SETGEN-001
tags:
  - cli
---

```gherkin
Feature: generic set

  Scenario: scalar and list edits
    Then assignedTo=, responsibility= and tags.add change one line each and keep comments

  Scenario: refusals
    Then an out-of-enum asilLevel and a dangling blockedBy are refused with the file unchanged

  Scenario: dry run and unknown fields
    Then --dry-run writes nothing and an unknown field lists the supported fields
```
