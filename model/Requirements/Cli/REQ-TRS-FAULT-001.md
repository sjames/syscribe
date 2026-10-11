---
type: Requirement
id: REQ-TRS-FAULT-001
name: "Failure modes of FMEA entries and fault-tree events can be injected into an executable composite"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - safety
  - fault-injection
---

An FMEA row or a fault-tree event shall be able to carry a machine-readable fault model, and scenarios of a composite shall be able to inject and clear it (GH #272, tracked in the fault-injection issue). Design: `docs/design/fault-injection.md`.

## Behavior

- **Fault model.** `injection:` is a list on an `FMEAEntry` row (in `entries:`) and on a `FaultTreeEvent`. The element's id is the fault's id. Each item has a `kind`:
  - `stuck` — `variable` of a part's machine is held at `value` (a stuck sensor): writes to it are discarded while the fault is active.
  - `offset` — `variable` is biased: every value written to it is increased by `by` (same dimension), and the current value is biased when the fault becomes active.
  - `silent` — the whole `part` fails silent (fail-stop): it handles no event, fires no timer, sends nothing, and its variables keep their values. It cannot be cleared.
  - `drop` — messages sent through `port` (optionally only `event`) are lost.
  - `delay` — messages sent through `port` (optionally only `event`) arrive `by` later.
  - `corrupt` — `field` of the payload of messages through `port` (optionally only `event`) is replaced by the value of `value`.
  - The target part is `part` (an instance name of the composite); when omitted, every part whose definition is, or specialises, the element's `ref`.
- **Static checks.** `E936` for a malformed `injection:` (not a list, unknown `kind`, a field missing for its kind, `value`/`by` that is not an expression or duration, `by` of a `delay` negative).
- **Run time.** A fault is inactive until injected. `System::inject(id)` activates every item of that element at the current virtual time and `clear(id)` removes it; both are recorded (id, time) and appear in the verdict's event list. Resolution against the composite happens when the system is built; a fault whose target part is not in the composite is ignored by it, and a fault that applies but does not fit (an unknown variable or port, a `value`/`by` of the wrong kind or dimension) is `E936` when a scenario injects it, without affecting the other scenarios. The resolved faults are part of the composite's evidence hash. Value faults stack, are removed independently, and apply to every write of the variable.
- **Scenarios.** `Given|When fault <id> is injected` and `When fault <id> is cleared` extend the composite vocabulary (REQ-TRS-SYSCOMP-001). `E937` when a step names an id that is not an `FMEAEntry` or `FaultTreeEvent` with an `injection:`.
- **Determinism and evidence.** No randomness; the same scenario gives the same trace. The recorded verdict lists every injection and clearing with its time.
- **Not in v1:** probabilistic or time-triggered faults declared on the element, the campaign over single faults and cut sets (phase 2), faults on plant internals other than through the variables they are bound to.
