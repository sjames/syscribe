---
id: REQ-TRS-MG-010
type: Requirement
title: refines link shall be honored on behavioral definitions (functional analysis refining requirements)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** honor the base-format `refines:` link ([[REQ-TRS-MG-001]]) on **behavioral
definitions** — `ActionDef`/`Action` and `StateDef`/`State` — not only on `UseCaseDef`/
`UseCase`. In MagicGrid the white-box functional analysis (W2 activities and state machines)
**refines** the system requirements it realises; today `refines:` is validated and indexed
only for use cases (`validator.rs` honours it solely on `UseCaseDef`/`UseCase`), so a W2
behavioral element carrying `refines:` is silently ignored (the gap surfaced building
`model_mg/`, whose W2 functions could not be traced to the W1 requirements they refine).

This requirement **extends** the existing `refines:` machinery to behavioral defs; it does
**not** add a new field or a new finding code.

### Behaviour

- `refines:` on an `ActionDef`, `Action`, `StateDef`, or `State` is resolved by qualified
  name or stable id (§11.10), exactly as for use cases. Each entry **shall** raise the
  existing **`E316`** when it resolves to nothing or to an element that is not a
  `Requirement`/`RequirementDef`.
- The **`refinedBy`** reverse index (§11.11) on each referenced requirement **shall** include
  the refining behavioral element alongside any refining use cases, and surface it in `show`.
- The **`W307`** "missing refines" warning **shall remain scoped to `UseCaseDef`** and is
  **not** extended to behavioral defs — not every action or state refines a requirement, so
  an absent `refines:` on a behavioral def is never a finding.

**Source:** MagicGrid white-box functional analysis (W2) refining system requirements (W1) —
gap identified building the `model_mg/` EV-charging-station model. Extends the `refines:`
field of [[REQ-TRS-MG-001]].

**Acceptance criteria:**

- An `ActionDef` with `refines: [REQ-…]` resolves the target, raises no finding, and the
  referenced requirement reports the action under its `refinedBy` index (visible in `show`);
  resolution by qualified name and by stable id both work.
- A `StateDef` whose `refines:` names an unresolved operand, or one resolving to a
  `PartDef`, raises `E316`.
- An `ActionDef`/`StateDef` with no `refines:` raises **no** `W307` (the missing-refinement
  warning stays use-case-only).
