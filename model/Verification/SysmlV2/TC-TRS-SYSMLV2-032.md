---
id: TC-TRS-SYSMLV2-032
type: TestCase
testLevel: L3
status: active
name: "Verify the MCP sysml_submodels tool is registered read-only and returns the same data as sysml --json without touching disk."
verifies:
  - REQ-TRS-SYSMLV2-032
sourceFile: repo:crates/syscribe/tests/sysml_inspect.rs
testFunctions:
  - mcp_sysml_submodels_matches_cli_json_and_is_read_only
---

```gherkin
Feature: sysml_submodels MCP tool (TC-TRS-SYSMLV2-032)

  Scenario: tool matches CLI JSON
    Given an MCP server over a model with a SysMLv2 submodel
    When a client lists tools and calls sysml_submodels
    Then the tool carries readOnlyHint, its result equals sysml --json, and the model directory is unchanged
```
