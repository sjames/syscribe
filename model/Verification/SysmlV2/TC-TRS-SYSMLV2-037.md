---
id: TC-TRS-SYSMLV2-037
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml writes SysML v2 text to stdout, a file and a directory, scopes to a package, rejects an unknown package and never modifies the model."
verifies:
  - REQ-TRS-SYSMLV2-037
sourceFile: repo:crates/syscribe/tests/sysml_export.rs
testFunctions:
  - cli_export_sysml_to_stdout_with_stderr_summary
  - cli_export_sysml_scope_and_unknown_scope
  - cli_export_sysml_out_file_and_dir_leave_model_untouched
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-037)

  Scenario: Tool shall provide an export-sysml command rendering the model or a package subtree as SysML v2 text
    Given a model with a package and two part defs
    When the user runs export-sysml, with a scope, with --out file and --out dir
    Then SysML text is produced accordingly, an unknown scope exits non-zero, and the model directory is unchanged
```
