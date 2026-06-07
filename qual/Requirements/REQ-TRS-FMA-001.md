---
id: REQ-TRS-FMA-001
type: Requirement
name: Feature Model Boolean Encoding
title: Tool shall construct the feature model tree and encode it as a deterministic propositional formula
status: draft
reqDomain: software
verificationMethod: test
---

The solver-backed analysis ([[REQ-TRS-FMA-002]]) operates on a propositional encoding of the feature model's Boolean layer. The tool **shall** build a rooted feature tree and translate it to a propositional formula as specified here. This encoding is the single source of truth shared by every deep analysis, so its correctness is the foundational obligation.

### Tree construction

- The tool **shall** allocate one Boolean variable per `FeatureDef`, identified by its qualified name.
- The **parent** of a `FeatureDef` is the enclosing feature `FeatureDef` (the `_index.md` group node of its directory), and **shall** be overridable by an explicit `parentFeature:`.
- The **root(s)** are the top-level `FeatureDef`s of the package named by a `Configuration.featureModel:`. The encoding **shall** introduce an implicit always-true root so that top-level features are treated uniformly as children of the root with their declared `groupKind`.
- The **children of a group** node (one whose `groupKind` is `alternative` or `or`) are the `FeatureDef`s whose parent is that node.

### Encoding rules

For each feature `N` with parent `P` (where `child ⇒ parent` means "N selected implies P selected"):

| Relationship | Clause(s) |
|---|---|
| Any non-root `N` | `N ⇒ P` |
| `N.groupKind: mandatory` | `P ⇒ N` (with the above, `N ⇔ P`) |
| `N.groupKind: optional` | (only `N ⇒ P`) |
| `N.groupKind: alternative` (group on `N`) | `N ⇒ exactly-one(children(N))`; each `child ⇒ N` |
| `N.groupKind: or` (group on `N`) | `N ⇒ at-least-one(children(N))`; each `child ⇒ N` |
| `cardinality: "m..n"` on an `alternative`/`or` group | `N ⇒ between-m-and-n(children(N))` (overrides the default count) |
| `requires: [B, …]` on `A` | `A ⇒ B` for each entry |
| `excludes: [C, …]` on `A` | `¬(A ∧ C)` for each entry |
| implicit root | root variable asserted **true** |

- Group-count constraints (`exactly-one`, `at-least-one`, `between-m-and-n`) **shall** be encoded faithfully (e.g. via pairwise/▢ commander encodings for SAT, or directly for a BDD).
- Unresolved `requires`/`excludes`/`parentFeature` targets are reported by the extensional pass (`E212`) and **shall** be skipped (not silently mis-encoded) here.

### Determinism

- Variable ordering **shall** be deterministic (e.g. features sorted by qualified name), so the encoding — and every downstream analysis — is reproducible across runs and platforms (see [[REQ-TRS-FMA-006]]).

**Source:** ADR-FM-001; spec §9.6 (`groupKind`, `cardinality`, `requires`/`excludes`), §9.9 (two-level structure).

**Acceptance criteria:** For a small hand-checkable model, the set of satisfying assignments of the encoding **equals** the hand-enumerated set of valid configurations. Each encoding rule (`mandatory`, `optional`, `alternative`/XOR, `or` with default and explicit `cardinality`, `requires`, `excludes`, root) has at least one positive and one negative fixture demonstrating it admits/rejects the expected assignments. The encoding of a fixed model is identical across two runs.
