---
id: REQ-TRS-SYSMLV2-090
type: Requirement
name: "Control node parameter bodies are ingested and exported"
status: verified
reqDomain: software
verificationMethod: test
---

The `in`/`out`/`inout` parameter declarations in the body of a `fork`, `join`, `decide` or `merge` node — in the standalone form and in `then fork f { ... }` — shall be ingested as the additive `parameters:` list of the node's `controlNodes:` entry (`{name, direction, typedBy?}`, spec 8.7.4), and `export-sysml` shall write them back as the node's body.

**Source:** `REQ-TRS-SYSMLV2-090` (product model), `ADR-SYS-SYSMLV2-001`.
