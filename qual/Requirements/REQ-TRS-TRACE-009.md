---
id: REQ-TRS-TRACE-009
type: Requirement
title: Tool shall detect and report cycles in supertype, derivedFrom, and subsets graphs
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** detect cycles in all directed hierarchical relationship graphs and emit a distinct error for each cycle type:

| Code | Relationship | Description |
|---|---|---|
| `E016` | `supertype:` | An element's specialisation chain forms a cycle |
| `E017` | `derivedFrom:` | A requirement's derivation chain forms a cycle |
| `E018` | `subsets:` | A feature's subsetting chain forms a cycle |
| `E107` | `typedBy:` | A usage's typing chain forms a cycle — **including the degenerate self-reference** (a usage typed by itself) |

Each cycle **shall** be reported at the element whose outgoing link closes the cycle. A model containing any of these cycles is **non-conformant**; the tool must not silently accept it. The **self-reference** case (a length-1 cycle, e.g. `typedBy: <self>` or `supertype: <self>`) is a cycle and **shall** be reported like any other — an element cannot be its own type or supertype.

**Source:** §11.12 (`E016`, `E017`, `E018`); `E107` closes the gap that `typedBy` was previously excluded from cycle detection (GH #25). `E107` is a structural cycle error, not a name-resolution error, so it is **not** suppressed under the `--config` projection lens.

**Rationale:** Cyclic hierarchies make qualified-name resolution and reverse-index computation diverge. They also violate the SysML specialisation, requirement-breakdown, and typing semantics, which require directed acyclic graphs. `typedBy` was the one hierarchical relationship whose cycles were silently accepted.

**Acceptance criteria:** For each of the four relationship types, a crafted model that introduces exactly one cycle produces exactly one finding with the corresponding code. A `typedBy: <self>` self-reference produces exactly one `E107`. Removing the cycle removes the finding; a valid acyclic model produces none of `E016`/`E017`/`E018`/`E107`.
