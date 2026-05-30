---
id: REQ-TRS-TRACE-009
type: Requirement
name: Acyclicity of Hierarchical Relationships
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

Each cycle **shall** be reported at the element whose outgoing link closes the cycle. A model containing any of these cycles is **non-conformant**; the tool must not silently accept it.

**Source:** §11.12 (`E016`, `E017`, `E018`)

**Rationale:** Cyclic hierarchies make qualified-name resolution and reverse-index computation diverge. They also violate the SysML specialisation and requirement-breakdown semantics, which require directed acyclic graphs.

**Acceptance criteria:** For each of the three relationship types, a crafted model that introduces exactly one cycle produces exactly one finding with the corresponding code. Removing the cycle removes the finding.
