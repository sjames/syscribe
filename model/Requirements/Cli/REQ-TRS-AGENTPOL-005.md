---
type: Requirement
id: REQ-TRS-AGENTPOL-005
name: "The evidence sidecar stays protected from agents except for the commands the policy names"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
---

`[agents] sidecar_writers` shall name the commands that may still write the evidence sidecar (`<model>/.syscribe/**`, except `scripts/`) when a `protected_paths` glob covers it, so recorded evidence cannot be hand-edited by an agent while an automated `ingest-results` keeps working (GH #469).

- Only an agent's change is affected, as with the rest of the policy. A command not listed stays refused for those paths; a malformed `sidecar_writers` is `W762` and grants nothing.
- The run history (`.syscribe/results-history.json`, `ingest-results --run`) is written through the same write layer as the sidecar, so `--dry-run` writes neither and `protected_paths` applies to it; a protected history is refused before the sidecar is written, so the two never fall out of step.
- The audit (`W760`) does not report files under `.syscribe/` (other than `scripts/`) when `sidecar_writers` is set: git cannot tell a listed command's write from a hand edit, so a hand-edited sidecar committed by an agent is accepted as a known limit. Every other protected path in the same commit is still reported.
- Every refused sidecar write exits 1 and names the rule (`protected_paths`) and the file, also under `--dry-run`.
- `policy`, `agent-guide` and the MCP instructions state which commands may write the sidecar, and `help policy` says that `.syscribe/**` should not be protected without it when ingest is automated.
