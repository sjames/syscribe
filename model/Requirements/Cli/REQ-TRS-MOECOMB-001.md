---
type: Requirement
id: REQ-TRS-MOECOMB-001
name: "MoE settings are checked, a MoE can combine measurements and other MoEs, and Celsius is a unit"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - magicgrid
---

The MagicGrid `trade-study` and the expression language shall reject silent misconfiguration and express multi-measurement effectiveness (GH #476, #477, #478).

- `MG034` (warning) reports an unknown `mg_moe*`/`mg_mop*` key with a "did you mean" hint; a missing `mg_moe_direction`/`threshold`/`objective`, `mg_moe_weight` (not asked of a MoE that is another's input) or `unit`/`measurement` (not asked of a MoE that combines variables or inputs); a threshold equal to the objective; weights that do not sum to 1 over the MoEs being ranked; an `mg_moe_inputs` entry that names no MoE, or a cycle of inputs; and an `expression:` name that no `mg_moe_variables`, `mg_moe_inputs` or Configuration `parameterBindings` key provides.
- `mg_moe_variables: {var: {measurement, aggregate?, unit?}}` binds expression variables to recorded measurements; `mg_moe_inputs: {var: <MoE qname or id>}` binds them to the score (0..1) of another MoE. Inputs are evaluated first, a cycle or unknown input leaves the cell `n/a` with the reason, and a knock-out of a measurement or an input propagates to the MoE with `input <name>: <reason>`. Every leaf is still reported; only MoEs that are nobody's input are weighted in the rollup.
- Temperatures keep readings and differences apart. `degC`/`degF` are absolute readings (`23.4 degC` is `296.55 K`); `delta_degC`/`delta_degF`/`delta_K` are differences; a plain `K` is neutral. A reading minus a reading is a difference; a reading plus or minus a difference is a reading; differences scale freely. Adding two readings, a reading and a plain `K` (ambiguous), scaling or negating a reading, `sqrt`/`pow`/`abs` of a reading, and comparing a reading with a difference are dimension errors saying what to write instead. `--unit degC` of a difference, or `--unit delta_degC` of a reading, is refused. `C` remains coulomb. The units are accepted in feature parameter `unit:` slots and `mg_moe_unit`.
