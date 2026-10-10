---
type: TestCase
id: TC-TRS-REQIFIMP-001
name: "import-reqif creates, deduplicates and updates requirements"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/reqif_import.rs
verifies:
  - REQ-TRS-REQIFIMP-001
tags:
  - cli
---

```gherkin
Feature: import-reqif

  Scenario: import
    Then each requirement object becomes a draft Requirement with name, text and the OEM id in extRef

  Scenario: idempotent re-import and update
    Then a second import creates nothing and --update rewrites only changed name and body

  Scenario: dry run, errors and round trip
    Then --dry-run writes nothing, malformed input exits 1 and an export-reqif file imports back
```
