---
id: TC-TRS-SYSMLV2-039
type: TestCase
testLevel: L3
status: active
name: "Verify names are quoted when not basic identifiers or when reserved."
verifies:
  - REQ-TRS-SYSMLV2-039
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export.rs
testFunctions:
  - identifiers_are_quoted_when_not_basic_or_reserved
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-039)

  Scenario: Tool shall emit valid SysML v2 identifiers, quoting non-basic names and reserved words
    Given names such as REQ-TRS-001, two words, 1abc, part and it's
    When they are rendered as SysML identifiers
    Then each is single-quoted with escapes and basic names stay bare
```
