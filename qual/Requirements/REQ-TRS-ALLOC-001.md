---
id: REQ-TRS-ALLOC-001
type: Requirement
title: Allocation supports two forms — allocatedTo-on-source (OSLC default) and the documented Allocation element — over one unified edge model
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support **two** ways to author an allocation, each first-class, sharing a
**single** edge model and a **derived** `allocatedFrom` reverse index:

1. **`allocatedTo:` on the source element** (the lightweight, OSLC-canonical default). The
   source element being allocated holds `allocatedTo: <target | [targets]>`; the source **is**
   the `allocatedFrom`, which is **derived**, not authored — obeying the OSLC link-direction
   rule (§12.1, [[REQ-TRS-TRACE-001]]) exactly like `satisfies`/`verifies`/`refines`.
2. **A standalone `Allocation` element** (a `type: Allocation` element naming both
   `allocatedFrom` and `allocatedTo`, top-level or per `features:` entry). This is a **reified
   relationship artifact** — kept for when the allocation itself needs a documented body
   (freedom-from-interference argument, deployment rationale, integration-test notes). Like an
   `ADR`, naming both endpoints is its purpose, not redundancy. The FFI/deployment allocations
   of §12.6 use this form.

**Neither form is being removed and there is no forced migration** — existing models keep
their standalone `Allocation` elements unchanged. Guidance: use `allocatedTo` by default;
promote to an `Allocation` element only when the allocation needs its own documentation.

### Derived `allocatedFrom` index

The tool **shall** compute a derived reverse index **`allocatedFrom`** on each target element
(the sources allocated to it), aggregating edges from **both** forms, surfaced in `show` /
`links` / the export `computed` block exactly like `verifiedBy` / `refinedBy` / `mopRefinedBy`.

### One unified edge extractor

The allocation-edge set **(source → target)** consumed by `MG041`, `MG081`, the
`matrix --allocations` view, and the derived `allocatedFrom` index **shall** come from a
**single** extractor that yields edges from both forms:

- form 1 — for every element holding `allocatedTo:`, `(that element → each allocatedTo target)`;
- form 2 — for every `type: Allocation` element, its top-level `allocatedFrom`×`allocatedTo`
  cartesian, **and** each `features:` entry carrying both `allocatedFrom` and `allocatedTo` —
  **regardless** of whether the entry also declares a feature-level `type: Allocation` (the
  form the validator previously required but `matrix --allocations` did not, the inconsistency
  that produced false `MG041`/`MG081`).

So `matrix --allocations` and the gate can never disagree, and an allocation authored in
either form produces the same edge.

### Resolution & redundancy

- Each `allocatedTo` operand **shall** resolve by qualified name or stable id; an unresolved
  target raises the existing `E503` (and `E502` for an unresolved `allocatedFrom` on the
  standalone form).
- When the **same** `source → target` edge is declared by **both** an `allocatedTo` on the
  source **and** a standalone `Allocation` element, the tool **shall** emit a warning (the
  duplicate is redundant — pick one form). A single edge in a single form raises nothing.

**Source:** allocation-extractor hardening + OSLC alignment — a false-positive class found
building the MagicGrid `model_mg/` model (the validator's `MG041`/`MG081` extractor required a
per-feature `type: Allocation` that `matrix --allocations` did not). Aligns the extractor of
[[REQ-TRS-MG-005]] / [[REQ-TRS-MG-014]] with the matrix of [[REQ-TRS-MG-006]], the
allocation-feature checks of [[REQ-TRS-VAL-009]] (`E500`–`E503`), and the OSLC direction rule
of [[REQ-TRS-TRACE-001]].

**Acceptance criteria:**

- A W2 `ActionDef` carrying `allocatedTo: <logical PartDef>` (form 1, no standalone element)
  **clears `MG081`**, the logical part **clears `MG041`** via the same link, the edge appears
  in `matrix --allocations`, and the target lists the action under its derived `allocatedFrom`
  in `show`.
- A legacy standalone `Allocation` element whose `features:` entry sets `allocatedFrom` +
  `allocatedTo` but **omits** the feature-level `type: Allocation` is still recognised as an
  edge (matrix + `MG041`/`MG081` agree), identical to a `type:`-tagged feature.
- The matrix / `MG041` / `MG081` edge set is identical whether an allocation is authored in
  form 1 or form 2.
- An `allocatedTo` naming an unresolved target raises `E503`.
- The same `source → target` edge declared in **both** forms raises the redundancy warning;
  one edge in one form raises nothing.
