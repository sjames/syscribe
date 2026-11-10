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
- Every refused sidecar write exits 1 and names the rule (`protected_paths`) and the file, also under `--dry-run`.
- `policy`, `agent-guide` and the MCP instructions state which commands may write the sidecar, and `help policy` says that `.syscribe/**` should not be protected without it when ingest is automated.
