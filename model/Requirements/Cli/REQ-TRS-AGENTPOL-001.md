---
type: Requirement
id: REQ-TRS-AGENTPOL-001
name: "The writing commands enforce the project's agent policy"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
---

Commands that change the model shall enforce the `[agents]` policy of `.syscribe.toml` on a change made by an agent (GH #442, `ADR-SYS-AGENTPOLICY-001`).

## Behavior

- The policy is opt-in: no `[agents]` table, no policy. Its keys are `protected_paths`, `human_only_status`, `require_before_done`, `require_by`, `actors` and `audit_limit`. A malformed entry is ignored on its own (and reported, `W762`); a status the element type does not have, or an unknown element type, is malformed.
- A change is an agent's when it names an actor (`--by`) and does not pass `--as-human`; every MCP write is an agent's. With `require_by`, a model-writing command naming neither `--by` nor `--as-human` is refused.
- An agent's change is refused (exit 1, nothing written, the rule and how a person proceeds named) when it would write a path matching `protected_paths` — across every file a `move` rewrites and in each operation of a batch —, set a status listed for the type in `human_only_status`, or mark a PlanningItem `done` while `require_before_done` is unmet (`evidence`: no resolving evidence, `E719`; `validate`: the change introduces an error). The same refusal is reported under `--dry-run`.
- Protected paths are also guarded in the write layer, so commands without their own check cannot write them.
- `--as-human` is recorded as a `Syscribe-As: human` commit trailer; it is a declaration, not a proof.
