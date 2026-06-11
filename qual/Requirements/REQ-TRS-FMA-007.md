---
id: REQ-TRS-FMA-007
type: Requirement
title: feature-check --deep shall produce minimal (MUS) explanations using batsat assumption cores
status: draft
reqDomain: software
verificationMethod: test
---

The explanations required by [[REQ-TRS-FMA-005]] (for `E223` void and `E224` dead) **shall** be computed with batsat's assumption-based unsat cores rather than ad-hoc deletion over the whole constraint set, and **shall** be **minimal**.

### Mechanism

- Each removable model constraint `c` (the labelled constraints of the encoding — `requires`, `excludes`, `mandatory`, group at-least / at-most, and the root assertion) **shall** be guarded by a fresh **selector** variable `s_c`: every clause of `c` becomes `(¬s_c ∨ clause)`, and all `s_c` are passed as **assumptions** (forced true) to a single `solve_limited` call.
- When the query is UNSAT, batsat's unsat core (the subset of failed assumptions) identifies the constraints in conflict. The structural `child ⇒ parent` implications are part of the base formula (not selector-guarded), since they are not authoring choices.

### Minimality

- The reported core **shall** be a **minimal unsatisfiable subset (MUS)**: removing any single cited constraint makes the query satisfiable. The solver-returned core **shall** be reduced to an MUS by deletion over the returned core (not the whole set), so it is both minimal and cheap.
- The explanation **shall** remain sound (the cited constraints, with the query's assumptions, are jointly unsatisfiable) and deterministic ([[REQ-TRS-FMA-006]]).

This is a mechanism/quality refinement of [[REQ-TRS-FMA-005]]; the observable explanation is the same kind of conflict set, now guaranteed minimal and computed via the solver.

**Source:** ADR-FM-002 (batsat assumption cores); refines REQ-TRS-FMA-005.

**Acceptance criteria:** For a void model whose conflict is `A requires B` together with `A excludes B`, the explanation lists exactly those two constraints, and removing either one makes the model satisfiable (demonstrating it is an MUS). Adding unrelated, satisfiable constraints to the model does **not** add them to the core. The explanation is identical across two runs.
