---
id: TC-TRS-SYSMLV2-031
type: TestCase
testLevel: L3
status: active
name: "Verify syscribe sysml reports files, per-kind element counts, unmapped counts and W540-W543 findings, as text and JSON, and exits 0 without submodels."
verifies:
  - REQ-TRS-SYSMLV2-031
sourceFile: repo:crates/syscribe/tests/sysml_inspect.rs
testFunctions:
  - cli_sysml_text_report
  - cli_sysml_json_report
  - cli_sysml_without_submodels_exits_zero
  - report_lists_files_kinds_unmapped_and_findings
  - report_json_shape_and_no_submodel_is_empty
---

```gherkin
Feature: SysMLv2 submodel inspection command (TC-TRS-SYSMLV2-031)

  Scenario: text and JSON report
    Given a submodel with a part def, a port def and an unmapped calc def
    When the user runs sysml and sysml --json
    Then the files parsed, PartDef/PortDef counts, calc def x1 and the W543 finding are reported

  Scenario: no submodel
    Given a model with no sysmlSubmodel package
    When the user runs sysml
    Then a message is printed (empty submodels array under --json) and the exit status is 0
```
