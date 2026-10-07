---
type: Requirement
id: REQ-TRS-TREX-004
name: "The export is available as the CLI command trace-export and the read-only MCP tool trace_export with the same options and document"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-TREX-000]
breakdownAdr: Decisions::TraceExportADR
tags:
  - traceability
  - export
---

A new CLI command `syscribe -m <root> trace-export [--config <C>] [--sort <order>] [--out <file>]` shall write the document to stdout (or `--out`), exit 0, and treat an unknown option as a usage error (named on stderr, nothing on stdout, exit 1), consistent with the other commands' option checks. A read-only MCP tool `trace_export` with parameters `config` and `sort` shall return the same document as its result. Both shall be documented: `prompts/help/trace-export.md` (registered in the help index and `docs/cli/index.md`) and the `render_diagram`-style entry in `prompts/help/mcp.md`; the authoring prompt's command list gains the command.

## Rationale

CI scripts call the CLI; agents call MCP; one computation serves both.

## Scope

- No HTML or Markdown rendering; the Requirement diagram and `matrix` remain the human views.
