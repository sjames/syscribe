---
type: Requirement
id: REQ-TRS-CONTEXT-001
name: "context returns one bounded briefing bundle for an element"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - mcp
---

`syscribe context <ref> [--depth N] [--max-bytes N] [--format text|markdown|json]` and the MCP tool `context` shall return, in one call, what is needed to understand an element and its verification state (GH #435).

## Behavior

- The bundle holds the element's id, type, status, summary, file and body; upstream and downstream neighbours to `--depth` (default 1), each with id, type, status, relationship and a one-line summary; the verifying tests with the latest verdict and the environment, build and time it ran on; for a `PlanningItem` its parent, children, `achieves`, evidence, blockers, claim, assignee and open questions; and the validation findings on the element and on the neighbours listed, errors first.
- The output never exceeds `--max-bytes` (default 16000, minimum 1024) in any format, and JSON output is valid at every bound. When it would exceed it, the least important items are dropped first (downstream, upstream, parents, findings, verification, planning, then the body) and a notice — `truncated` in JSON — names what was omitted and how many.
- An unknown reference exits 1 with a suggestion; a bad option or value exits 2. The MCP tool returns the same JSON and is read-only.
