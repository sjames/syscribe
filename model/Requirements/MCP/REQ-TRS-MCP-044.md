---
type: Requirement
id: REQ-TRS-MCP-044
name: "coverage and coverage_matrix share one classifier; draft-only links are planned, not verified"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The `coverage` and `coverage_matrix` tools shall derive their verdicts from a **single** per-cell
classifier, so they can never contradict each other for the same requirement. `coverage` is the
row-collapse of the same Requirement × Configuration grid `coverage_matrix` displays.

## Per-cell classifier (requirement × configuration)

- `na` — the requirement's effective `appliesWhen` is false here (excluded from the denominator).
- `gap` — active, and **no** verifying TestCase runs here.
- `planned` — active; a verifying TestCase runs here but it is **draft** (verification intent, not
  done).
- `covered` — active; a **non-draft** TestCase runs here; result unknown.
- `passing` / `failing` — `covered` and the latest ingested verdict is pass / fail.

## Global rollup (`coverage`)

For each requirement, collapse its row over the configurations where it is active (logical **AND**
— verified only where covered everywhere it applies):

- **verified** — every active cell is `covered`/`passing`.
- **planned** — at least one active cell is `planned` and none is `gap`/`failing`.
- **unverified** — any active cell is `gap` or `failing`.
- not-applicable — active in no configuration (never silently counted as verified).

## Invariant

A requirement that is `gap` or `planned` in **every** configuration where it is active shall
**not** appear in `coverage`'s verified set. (Supersedes the draft-agnostic verified rule of
REQ-TRS-MCP-019; `coverage_matrix` continues REQ-TRS-MCP-036 with the added `planned`/`failing`
cell states.)
