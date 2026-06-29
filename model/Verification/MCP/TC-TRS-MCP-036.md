---
type: TestCase
id: TC-TRS-MCP-036
name: "ingest_results dry-run/commit join external verdicts under guard"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-035
tags:
  - mcp
  - write
---

```gherkin
Feature: Result ingestion

  Scenario: dry-run reports the verdict delta without writing
    Given a fixture whose TC-FX-001 declares a test function
    When ingest_results is called with an inline cargo-json report marking that function passed, dry_run defaulting true
    Then the response reports the verdict delta and writes no sidecar

  Scenario: commit writes the sidecar
    When ingest_results is called with dry_run=false for the same report
    Then written is true and a .syscribe/results.json sidecar exists

  Scenario: a malformed report errors and leaves the sidecar unchanged
    When ingest_results is called with unpar_seable content and dry_run=false
    Then a structured error is returned and no sidecar is written
```
