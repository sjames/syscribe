---
type: Requirement
id: REQ-TRS-SMIR-001
name: "State machines are lowered to a typed, resolved intermediate representation that every execution feature consumes"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

Between the YAML model and the interpreter there shall be one intermediate representation (IR), produced by a deterministic *lowering* pass, so that the semantics, the static checks, scenarios, coverage, mutation and export all read the same resolved machine (GH #275, part of #272). There is no bytecode virtual machine: the IR is executed by a tree-walking interpreter, and its design leaves room for another back end.

## Behavior

- **Lowering** turns an executable `StateDef` into a `Machine`: states in a flat table with ids, parent, kind (atomic, compound, parallel, final), initial child and entry/exit action lists; transitions with id, source, target (or internal), trigger (`event`, `timer`, `change`, `done`), guard, and an ordered action list; a variable table with slots, declared type, dimension and initial value; an event table. Document order is preserved as the transition priority. Every IR node records its **source** (qualified name and YAML path), so a trace, a coverage hole or a mutant points back at the model text.
- **Expressions are resolved and typed** during lowering: each name becomes a variable slot, an event field, a model attribute constant or an enumeration literal; each node gets a static type and dimension. Unbound names, dimension errors in constant positions and an `after` that is not a duration are therefore reported at lowering time, once, with the checks of REQ-TRS-SMSCHEMA-001 (`E882`–`E885`, `W933`) as its diagnostics — not discovered mid-run.
- **The IR is a value:** serialisable to a canonical JSON form (stable ordering, no addresses), printable with `syscribe compile <StateDef> [--json]`, and hashable. Its hash is recorded with every recorded scenario result and trace, so evidence says exactly which machine produced it and a changed machine is detectable.
- **One semantics:** the interpreter of REQ-TRS-SMEXEC-001, the coverage instrumentation, the mutation operators (which transform the IR, never the model files) and the SCXML exporter all work from the IR; none re-reads the YAML.
- **No hidden state:** an IR plus an event/time script determines a run completely.
- Lowering is read-only and deterministic; its own tests include a golden IR for the worked examples of `docs/model-guide/state-machines.md`.
