---
type: Requirement
id: REQ-TRS-AGENTPOL-003
name: "The policy is stated where agents read"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
---

The effective policy shall be inspectable and shown to the agents it governs (GH #442).

## Behavior

- `syscribe policy [--json]` prints the rules in plain words, any ignored entries, and whether the current invocation is treated as an agent's or a person's; with no `[agents]` table it says so.
- `agent-guide` gains a `policy` section, and the MCP `initialize` instructions a line, stating the rules; both are absent when no policy is configured.
