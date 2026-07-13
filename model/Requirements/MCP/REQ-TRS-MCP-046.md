---
type: Requirement
id: REQ-TRS-MCP-046
name: "MCP exposes release baselines through read-only tools"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - baseline
---

The MCP server shall expose release baselines (`ADR-SYS-BASELINE-001`; `REQ-TRS-BL-*`) so an
LLM agent can inventory releases, compare them, and confirm their integrity.

## Read tools

- `baseline_list` — report every `Baseline` element with its id, name, status, date, scope
  summary, `gitTag`/`gitCommit`, `elementCount`, and `aggregateHash`. Read-only;
  deterministic ordering.
- `baseline_diff` — given two baseline ids, report the element-level `added` / `removed` /
  `changed` sets (keyed by stable id, grouped by type) computed from the two manifests, plus
  whether the aggregates are identical. Read-only.
- `baseline_verify` — given a baseline id (or all), report the content proof (recomputed
  aggregate vs seal vs manifest) and git tag↔commit consistency, with a boolean `passed`.
  Read-only.

All three tools shall be annotated `read_only_hint = true`, share the same seal/scope/diff
logic as the CLI (so the two surfaces never disagree), and account for configuration-projected
scope (REQ-TRS-BL-011).

## No `baseline_create` tool

Sealing a baseline is deliberately **not** exposed as an MCP write tool. Unlike the element
write tools (which stage a throwaway model copy), `baseline create` is a source-control
action: it captures the live `HEAD`, requires a clean working tree, and writes a committed
manifest — none of which is meaningful against a staged copy. Creating a release is a
deliberate human/CI step performed via the CLI; the MCP surface is read-only.
