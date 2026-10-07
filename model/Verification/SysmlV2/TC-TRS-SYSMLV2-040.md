---
id: TC-TRS-SYSMLV2-040
type: TestCase
testLevel: L3
status: active
name: "Verify unsupported elements become skipped comments and per-type counts are reported."
verifies:
  - REQ-TRS-SYSMLV2-040
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export.rs
testFunctions:
  - unsupported_elements_are_commented_and_counted
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-040)

  Scenario: Tool shall comment out unsupported elements as skipped and report exported and skipped counts
    Given a model with a TestCase beside supported elements
    When it is exported
    Then a skipped comment names the TestCase and the report counts exported and skipped types
```
