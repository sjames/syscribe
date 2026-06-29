---
type: Requirement
id: REQ-TRS-MCP-035
name: "ingest_results joins external test verdicts into the model under guard"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
  - write
---

The MCP server shall expose an `ingest_results` guarded-write tool that parses an external test
report into per-test verdicts and updates the results sidecar, with semantics identical to the CLI
`ingest-results`.

## Behaviour

- Inputs: `{ format?: "cargo-json" | "junit", path?: string, content?: string, dry_run?: boolean }`;
  exactly one of `path`/`content` is supplied (`content` allows inline reports); `format` may be
  inferred from a `path` extension.
- `dry_run` (default true) returns the verdict delta (TestCases whose verdict flips) and the `W010`
  findings that would appear or clear, without writing the `.syscribe/results.json` sidecar.
- A commit (`dry_run: false`) writes the sidecar, returns the committed delta and `written: true`,
  and rebuilds the in-session store so the other evidence tools observe the new verdicts.
- A malformed or unrecognised report fails with a structured error and leaves the sidecar unchanged.
