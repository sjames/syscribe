---
id: REQ-TRS-INV-001
type: Requirement
name: Tool shall check named safety properties with degraded clauses and temporal operators
status: draft
reqDomain: software
verificationMethod: test
---

A composite scenario may name a property once (`Where <name> is <expression>`), replace it for the fault sets that make part of it impossible (`Except under <fault> [and <fault>…]: <expression>`), and check a property over time (`Then within <duration>: …`, `Then always …`, `Then from now on …`, `Then never …`) (spec §8.12.8).

The tool **shall**:

- judge `within <d>` as reached at a settled instant no later than `d` after the step and kept at every settled instant after it;
- judge `always`/`from now on` and `never` at every settled instant from the step to the end of the run;
- judge a named property by the full expression unless the faults injected now include every fault of an `Except under` clause; of several such clauses use the one whose faults include those of every other, and otherwise report "no applicable degraded clause" as an error, not a defeat;
- report a scenario that passed only in a degraded clause with that clause, and a fault set whose invariants survived only degraded as "survived (degraded)" in `inject`, its report and its record;
- report malformed `Where`/`Except` lines, a step that names an undeclared property and an operator outside a `Then` step as `E886`, and an `Except` fault that is not a fault of the model as `E937`.

**Source:** GH #342, from the grid-tie inverter model (`model_gti/`): the invariants had to be weakened to "not (pwm on and both relays closed)" because a welded relay makes the safe state impossible, and they were checked once, at the end.

**Acceptance criteria:**

- A property that holds only after 2.2 s fails `within 2 s` and passes `within 2.5 s`; one reached at 1 s and lost at 1.5 s fails `within 2 s`.
- `always` fails on a violation that recovers.
- A welded element uses its clause and the set is "survived (degraded)"; two welded elements without a clause of their own are "no applicable degraded clause".
