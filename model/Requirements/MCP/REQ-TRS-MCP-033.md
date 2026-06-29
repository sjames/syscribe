---
type: Requirement
id: REQ-TRS-MCP-033
name: "run_report passthrough exposes the read-only report/analysis family under a guard"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - security
---

The MCP server shall expose a single guarded `run_report {command, args?, format?}` tool that
runs an allowlisted, read-only report/analysis CLI command against the served model and returns
its output, so an LLM can obtain the safety/security and other analysis reports
(`metrics`, `cyber-risk`, `safety-case`, `fmea`, `fault-tree`, `zones`, `conduits`,
`co-analysis`, `sbom`, `behavioral-coverage`, plus `audit`, `matrix`, `n2`, `trade-study`,
`verification-depth`, `testplan`, `magicgrid`, `impact`, `lint-docs`) without a dedicated tool
per report.

## Allowlist (read-only only)

- Only commands on a fixed server-side allowlist of read-only report/analysis commands may be
  run. Any other command — every write/authoring command and every file-emitting command
  (`render`, `plantuml`, `diagram`, `build-config`, `export-reqif`, `move`, `ingest-results`,
  `scaffold-gherkin`), and the `mcp` subcommand itself — shall be refused with a clear error and
  shall not be executed.

## Confinement

- The model root passed to the command shall be fixed to the server's own resolved root; a
  caller shall not be able to redirect it. Any caller-supplied argument that sets the model
  (`-m`/`--model`), redirects output to a file (`-o`/`--output`), or contains a filesystem path
  escaping the model shall be rejected.
- The command shall be invoked with a direct argument vector (no shell), so argument values
  cannot inject additional commands.

## Result

- Returns `{command, args, output, exitCode}`. The tool is read-only and shall be annotated
  `readOnlyHint: true`.
- When `format: "json"` is requested and the underlying command supports it, the JSON form of
  the report shall be returned.
