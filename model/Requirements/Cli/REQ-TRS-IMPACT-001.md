---
type: Requirement
id: REQ-TRS-IMPACT-001
name: "change-impact says which Configurations a proposed edit would affect, without writing"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe change-impact <element> [--set <field>=<yaml>]... [--param <name> --param-set <field>=<yaml>]... [--json]`, the MCP tool `change_impact` and the LSP request `syscribe/changeImpact` shall report which Configurations change validity or projection if the proposed edit to a feature, constraint or parameter range were made (GH #423).

- The edit is applied to a candidate copy by the guarded-write engine in dry-run mode; the model on disk is never touched.
- Per Configuration: errors before and after (element-level findings of its variant and its feature-model validity), whether it became invalid or valid, the new and resolved errors, and the elements added to or removed from its projection. Findings are reported under the real model root, not the candidate's.
- The payload carries the unified diff and the element's file and line, and has the published `change-impact` schema. `diff --config A --config B --json` also lists each differing element with its location (`changes`), additively.
