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
- `sidecar_audit` (`exempt`, the default, or `command`; anything else is `W762` and `exempt`) says how the audit (`W760`) treats a commit of a file under `.syscribe/` (other than `scripts/`) when `sidecar_writers` is set. `exempt` never reports it: git cannot tell a listed command's write from a hand edit, so a hand-edited sidecar committed by an agent is an accepted limit. `command` reports it unless the commit's `Syscribe-Command` trailer (written by `--commit`) names a listed writer, so a plain `git commit` of the sidecar is reported; the trailer is commit-message text, so this is a guard rail, not proof. Every other protected path in the same commit is reported in both modes.
- A `[results] runs_dir` moved out of `.syscribe/runs` is part of the evidence sidecar: a command in `sidecar_writers` may write it (the directory itself included), the refusal hint and `policy` apply to it, and `sidecar_audit` governs commits of it, including a directory inside the repository but outside the model, which the audit adds to the history it reads. It is not protected unless `protected_paths` names it; a directory outside the repository is guarded when written but not audited.
- An agent that is refused is told what to do: a refused sidecar write names `sidecar_writers` (or the commands that may write it), a `W760` raised under `command` names `--commit` and `sidecar_audit`, and `policy` states the audit mode in words. `help policy` holds copy-paste configurations and a message-to-fix table, `help tasks` and `help ingest-results` point to it.
- Every refused sidecar write exits 1 and names the rule (`protected_paths`) and the file, also under `--dry-run`.
- `policy`, `agent-guide` and the MCP instructions state which commands may write the sidecar, and `help policy` says that `.syscribe/**` should not be protected without it when ingest is automated.
