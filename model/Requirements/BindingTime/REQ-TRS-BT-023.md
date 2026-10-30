---
type: Requirement
id: REQ-TRS-BT-023
name: "params export writes a YAML values file for the parameters bound at a stage"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`params export --config X [--stage S] [--out FILE]` shall write a YAML file holding, for every parameter and feature toggle whose window contains the stage (default: `X`'s `bindingStage:`), its effective value for `X` (own, inherited, resolved, then `default:`), keyed by stable `FEAT-*` id and then parameter name, with unit and range as trailing comments.

**Acceptance criteria:**
- Exporting a `load`-stage Configuration yields one entry per `load`- or `runtime`-window item and none for `compile`-only items.
- Entries are keyed by `FEAT-*` id, not qualified name.
