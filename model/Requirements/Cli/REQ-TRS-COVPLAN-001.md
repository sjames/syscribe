---
type: Requirement
id: REQ-TRS-COVPLAN-001
name: "coverage tree honours the --plan and --config lenses"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - coverage
---

`syscribe coverage tree <req>` shall accept `--plan <TP-id>` and `--config <id>` like `matrix --rollup` (GH #252).

## Behavior

- `--plan TP-X` evaluates the tree on the plan lens of REQ-TRS-PLAN-006: only the plan's in-scope requirements and effective TestCases exist, so a test outside the plan does not count and a requirement outside it is absent from the tree. `--config C` projects onto the Configuration first; both compose (plan first, then projection).
- A root that is outside the lens exits 1 with the usual "does not resolve to a requirement" message naming the lens. An unknown plan or configuration exits 1 as for the other commands.
- Without either option the output is unchanged.
