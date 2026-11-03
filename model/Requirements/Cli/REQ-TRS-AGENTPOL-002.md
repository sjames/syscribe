---
type: Requirement
id: REQ-TRS-AGENTPOL-002
name: "validate reports policy violations that reached the files by another route"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
---

`validate` shall report, from git history, changes that broke the agent policy without passing the writers' checks (GH #442, `ADR-SYS-AGENTPOLICY-001`).

## Behavior

- A commit is agent-made when it carries a `Syscribe-Actor` trailer (and no `Syscribe-As: human`) or its author matches an `[agents] actors` glob. `validate` inspects the last `audit_limit` (default 200) commits that touched the model.
- `W760`: an agent-made commit changed a protected path. `W761`: an agent-made commit set a human-only status on an element of that type. `W762`: a malformed `[agents]` entry. All are warnings; `--deny` gates them.
- A commit marked `Syscribe-As: human`, or made by someone who is not an agent, raises nothing. With no `[agents]` table, or no git repository, nothing is audited.
