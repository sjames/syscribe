---
type: Requirement
id: REQ-TRS-DE-001
name: "A single guarded-write engine in syscribe-model is shared by MCP and the web server"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
  - mcp
---

The guarded-write engine currently implemented only inside the MCP server
(`crates/syscribe/src/mcp/{mod.rs,write.rs}`: path confinement, referential-integrity scan,
candidate-copy/apply/re-validate/diff, and the commit gate) shall be extracted into
`syscribe-model` so that it operates on a model root path and element list rather than MCP's
JSON tool types, and shall be the single implementation used by the MCP tools, the CLI, and any
new `syscribe-server` write endpoint — including the diagram editor's.

A single `patch_frontmatter` helper (split frontmatter, mutate the parsed YAML mapping, reassemble
the file, preserving unknown keys and the body) shall replace the independent copies of that
pattern currently duplicated across `mcp/mod.rs::apply_update`/`plan_create` and
`syscribe-server/routes/write.rs::put_element`/`patch_layout`.

## Regression requirement

The existing `crates/syscribe/tests/mcp_*.rs` integration test suite shall pass unmodified after
the extraction, proving the MCP tools' observable behavior (arguments, dry-run semantics,
validation-delta shape, commit gating) is unchanged.
