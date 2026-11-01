---
id: REQ-TRS-BT-023
type: Requirement
name: "params export writes a YAML values file for the parameters bound at a stage"
status: draft
reqDomain: software
verificationMethod: test
---

`params export --config X [--stage S] [--out FILE]` shall write a YAML file holding, for every parameter and feature toggle whose window contains the stage (default: `X`'s `bindingStage:`), its effective value for `X` (own, inherited, resolved, then `default:`), in two sections — `features:` (one boolean per feature toggle) and `parameters:` (stable `FEAT-*` id, then parameter name) — with unit, range and source as trailing comments. A feature takes part as a toggle when it is a real choice (not abstract or mandatory); a parameter when its feature is selected. A parameter `derivedFrom:` an expression is the system's to compute and is not in the file.

**Acceptance criteria:**
- Exporting a `load`-stage Configuration yields one entry per `load`- or `runtime`-window item and none for `compile`-only items.
- Entries are keyed by `FEAT-*` id, not qualified name.
