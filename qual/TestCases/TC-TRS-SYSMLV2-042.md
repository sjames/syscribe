---
id: TC-TRS-SYSMLV2-042
type: TestCase
testLevel: L3
status: active
name: "Verify the export_sysml MCP tool is registered read-only and returns the CLI text without touching disk."
verifies:
  - REQ-TRS-SYSMLV2-042
sourceFile: repo:crates/syscribe/tests/sysml_export.rs
testFunctions:
  - mcp_export_sysml_matches_cli_and_is_read_only
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-042)

  Scenario: MCP server shall expose a read-only export_sysml tool returning the SysML v2 text
    Given an MCP server over a model
    When a client lists tools and calls export_sysml
    Then the tool carries readOnlyHint, its text equals the CLI output, an unknown package is an error and the model directory is unchanged
```
