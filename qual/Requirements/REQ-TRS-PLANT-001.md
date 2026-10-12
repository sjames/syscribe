---
id: REQ-TRS-PLANT-001
type: Requirement
name: Tool shall run continuous plants written in the model in-process
status: draft
reqDomain: software
verificationMethod: test
---

A `PlantDef` declares `parameters:`, `inputs:`, `outputs:`, `states:` and `equations:` (`der(x) = expression`, `y := expression`) in the expression language; a part becomes a plant with `plant:` naming a `PlantDef` or a library plant (spec §8.8.6).

The tool **shall**:

- integrate the plant in-process at the composite's `plantStep:`, with `rk4` (default) or `euler`, inputs held over each step and sub-steps no longer than `maxStep:`, needing no external toolchain or build feature;
- give the same trace, bit for bit, for the same model and scenario on every run and platform;
- check every equation for kind and unit dimension (`E935`) and a `PlantDef`'s structure (`E938`) before anything runs;
- resolve `plant:` to a `PlantDef` or one of the library plants `Gain`, `FirstOrderLag`, `Integrator`, `Saturation`, `DelayLine`, `ImpedanceDivider`, with parameter overrides of the matching dimension (`E939`);
- refuse a composite with an in-process plant part and no `plantStep:` (`E934`), and a binding that does not read a plant output or write a plant input of the matching dimension (`E933`);
- apply `stuck` and `offset` faults to plant outputs, `silent` to a plant, and the `parameter` fault kind to a plant parameter until it is cleared (`E936`);
- add `exp`, `ln`, `sin`, `cos` and `tanh` over a dimensionless argument to the expression language.

**Source:** GH #340, from the grid-tie inverter model (`model_gti/`): its anti-islanding check could not be executed, because a state machine cannot model how the grid answers a perturbation, so the sensing common cause stayed a single cut set.

**Acceptance criteria:**

- A first-order lag driven by a unit step reaches 1 − e⁻⁵ after five time constants, and two runs give identical traces.
- A parameter fault and a stuck output change the response as specified.
- Each malformed `PlantDef` and `plant:` gives its code.
