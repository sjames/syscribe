---
type: Requirement
id: REQ-TRS-CONTRACT-001
name: "The PLE --json forms are versioned contracts with published schemas"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

The `--json` form of `features`, `feature`, `configure`, `residual`, `matrix`, `diff`, `feature-check` and `params check` shall carry a `schemaVersion` and conform to a JSON Schema printed by `syscribe schema <name>` (GH #413).

- Each command's real output is validated against its schema in CI, and a validator that accepts a wrong document fails the suite.
- A new optional property is compatible; removing or retyping one, or changing its meaning, bumps `schemaVersion` and is listed in `docs/releases/json-contract.md`.
- `params check` accepts `--json` and keeps its exit codes. `schema` needs no model; an unknown name exits 1 with a suggestion.
