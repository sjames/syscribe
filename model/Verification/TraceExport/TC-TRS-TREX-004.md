---
id: TC-TRS-TREX-004
type: TestCase
testLevel: L3
status: active
name: "Verify the trace-export CLI writes to --out (creating parents, nothing on stdout), rejects an unknown option before the model loads, has a registered help page, and that the MCP trace_export tool returns the same document with sort and config honoured and invalid values as tool errors."
verifies:
  - REQ-TRS-TREX-004
sourceFile: repo:crates/syscribe/tests/trace_export.rs
testFunctions:
  - out_writes_the_file_and_creates_parents
  - an_unknown_option_is_rejected_before_the_model_loads
  - help_page_is_registered
tags:
  - traceability
  - export
  - cli
  - mcp
---

Black-box CLI tests in `crates/syscribe/tests/trace_export.rs`, run with
`cargo test -p syscribe --test trace_export`, on the `TC-TRS-TREX-001` model. The MCP side is
covered by `trace_export_returns_the_same_document_as_the_cli` in
`crates/syscribe/tests/mcp_read.rs` (`cargo test -p syscribe --test mcp_read`), which drives a
real `syscribe mcp` subprocess against the fixture model under
`crates/syscribe/tests/fixtures/model` and compares the tool result with the CLI document.

```gherkin
Feature: the trace-export CLI and MCP surfaces (TC-TRS-TREX-004)

  Scenario: --out
    When the tool runs trace-export --sort asc --out <dir>/nested/deeper/trace.json
    Then the file is written, its parents created, stdout is empty
    And the file holds the bytes stdout would carry

  Scenario: an unknown option
    When the tool runs trace-export --bogus against a nonexistent model root
    Then it exits 1 naming the option and the valid ones (--config, --sort, --out)
    And --sort with no value exits 1 saying it expects a value

  Scenario: the help page
    When the tool runs help trace-export and trace-export --help
    Then both print the man page with its SYNOPSIS

  Scenario: the MCP tool
    When the client calls trace_export {} on the fixture model
    Then the result equals the CLI document, with full qualified names and coverage
    And {sort: "asc"} orders the requirements by qualified name
    And {config: "CONF-FX-001"} records the configuration
    And {sort: "random"} and {config: "CONF-NOPE-001"} are tool errors
```
