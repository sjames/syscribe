---
type: Requirement
id: REQ-TRS-MCP-039
name: "lint_docs scans prose and SVG for unresolvable model references"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `lint_docs` read tool that scans supplied `.md`/`.svg` files or
directories (recursively) for unresolvable model references, returning findings identical to CLI
`lint-docs --json`.

## Behaviour

- Findings shall use the canonical codes: `W099` (stable-ID token in prose), `W100` (qualified name
  in a Mermaid block), `W101` (SVG `sysml:ref`), `W102` (missing local image/embed). Remote URIs are
  treated as external and not flagged.
- Inputs: `{ paths: string[], codes?: string[] }`; `codes` filters the returned set. Each finding
  shall include `{ file, line, code, token|ref|path }`.
