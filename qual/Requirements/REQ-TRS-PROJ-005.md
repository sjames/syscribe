---
id: REQ-TRS-PROJ-005
type: Requirement
title: Tool shall provide all-configs validation, dead-element and aggregate-coverage detection, and variant diff
status: draft
reqDomain: software
verificationMethod: test
---

Beyond a single lens, the tool **shall** provide checks that range over the configuration family.

### Validate-all-configurations gate

- `validate --all-configs` **shall** run the lens validation ([[REQ-TRS-PROJ-002]], [[REQ-TRS-PROJ-003]]) for **every stored `Configuration`** and summarise the per-variant outcome (pass / finding counts).
- It **shall** exit non-zero if **any** configuration has an error-severity finding — the CI gate that every shipped product validates.
- `--json` **shall** emit a per-configuration result list.

### Dead elements

- `feature-check --deep` **shall** report (`W021`, warning) any element whose `appliesWhen:` is **unsatisfiable** given the feature model — i.e. it is active in **no valid configuration** (the element-level analog of a dead feature; SAT check `Φ ∧ appliesWhen(element)` UNSAT).

### Aggregate coverage

- `feature-check --deep` (or `--all-configs`) **shall** report (`W022`, warning) any requirement that is **active in at least one** stored `Configuration` but **covered in none** — a family-wide coverage gap distinct from the per-configuration `W015`.

### Variant diff

- `diff --config A --config B` **shall** report the elements, requirements, and tests **active in A but not B**, and **active in B but not A** (text and `--json`). It is an informational utility (no error semantics).

All four are dormant when no feature model is present.

**Source:** ADR-PROJ-001.

**Acceptance criteria:** `validate --all-configs` lists each stored configuration with its outcome and exits non-zero iff some variant has an error; an element whose `appliesWhen` is unsatisfiable under the feature model is reported `W021`; a requirement active in some configuration and covered in none is reported `W022`; `diff --config A --config B` lists the symmetric difference of active elements between A and B.
