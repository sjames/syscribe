---
type: Requirement
id: REQ-TRS-PIQ-004
name: "MCP exposes questions and a guarded question write tool"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

The MCP server shall expose the open questions and a guarded way to change them (GH #436).

## Behavior

- A read tool `questions {open?, status?, item?}` returns the same rows as the CLI.
- A guarded write tool `question {item, op: add|answer|defer, ...}` goes through the shared guarded-write engine (dry-run by default, validation delta, referential-integrity gate) and is hidden under `--read-only`.
- The server instructions tell a client to record a question instead of guessing when blocked on a decision.
