---
type: Requirement
id: REQ-TRS-BT-002
name: "A project may declare additional binding-time stages in [bindingTimes] and the order is merged"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A `[bindingTimes.<name>]` table in `.syscribe.toml` shall declare a stage with the keys `description`, `binder` (a free label for who resolves it) and `after:` / `before:` naming a neighbouring stage; keys are camelCase or snake_case. Declared stages shall be merged into the default order, so a project can refine the triad.

**Acceptance criteria:**
- A `deploy` stage with `after: compile` and `before: load` yields `compile < deploy < load < runtime`.
- A stage with no `after:`/`before:` is rejected as unplaceable (see [[REQ-TRS-BT-003]]).
