---
type: TestCase
id: TC-TRS-MCP-010
name: "Excluded report/render and feature-model commands are not advertised as tools"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-010
tags:
  - mcp
---

Verifies that the curated tool surface excludes the report/render family and the
feature-model/projection commands.

```gherkin
Feature: Curated tool surface

  Scenario: the cut-list commands are absent from tools/list
    Given an initialized mcp server over a fixture model
    When a tools/list request is sent
    Then the curated read and write tools are present
    And none of export, plantuml, render, n2, matrix, fmea, sbom, reqif, audit, metrics, zones is listed
    And none of feature-check, configure, diff is listed
```
