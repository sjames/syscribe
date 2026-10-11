---
type: TestCase
id: TC-TRS-SMSCXML-001
name: "export-scxml writes well-formed SCXML for the machine IR and crosscheck compares an external engine"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/sm_scxml.rs
verifies:
  - REQ-TRS-SMSCXML-001
tags:
  - state-machine
---

```gherkin
Feature: SCXML export

  Scenario: document
    Then the output is well-formed SCXML with a datamodel of SI base-unit numbers and deterministic text

  Scenario: mapping
    Then after, when, event, done, internal, assign and guard map onto SCXML as specified, change triggers using armed flags

  Scenario: no silent loss
    Then constructs with no SCXML form are written as comments and returned as notes

  Scenario: crosscheck
    Then an agreeing engine passes and a differing, failing or malformed engine is reported with exit 2
```
