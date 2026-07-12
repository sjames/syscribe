---
type: TestCase
id: TC-TRS-MCP-046
name: "MCP suspect_list reports links and suspect_accept baselines under the write guard"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_suspect.rs
verifies:
  - REQ-TRS-MCP-045
tags:
  - mcp
  - suspect-links
---

Verifies the MCP suspect-link tools: `suspect_list` surfaces suspect and unbaselined links,
and `suspect_accept` baselines a link under the common write guard (dry-run default,
validation delta with the resolved W090, commit + reload, refusal on a non-referenced
target, and refusal on a read-only server).

```gherkin
Feature: MCP suspect-link tools

  Scenario: suspect_list surfaces an unbaselined link
    Given a model with a trace link that has no baseline
    When the suspect_list tool is called
    Then the link is reported as unbaselined with its source, target, and kind

  Scenario: suspect_accept dry-run computes but does not write
    Given a source S linking target T
    When suspect_accept is called without dry_run:false
    Then written is false and disk is byte-for-byte unchanged
    And the response includes a diff and a validation delta

  Scenario: suspect_accept commit baselines the link and clears W090
    Given a stale (suspect) baselined link that emits W090
    When suspect_accept is called with dry_run:false
    Then written is true and the source gains the current baseline
    And the validation delta lists the W090 among resolvedWarnings

  Scenario: Accepting a non-referenced target is refused
    Given S has no trace link to target X
    When suspect_accept is called for S and X
    Then written is false and no file changes

  Scenario: A read-only server refuses suspect_accept
    Given the server was started with --read-only
    When suspect_accept is called
    Then the tool returns an error and writes nothing
```
