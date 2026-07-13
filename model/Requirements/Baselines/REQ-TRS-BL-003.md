---
type: Requirement
id: REQ-TRS-BL-003
name: "Baseline scope selection"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - scope
---

A baseline's `frozenScope` shall define which elements are frozen. In v1 the selector shall
support:

- `package` — a qualified-name prefix; only elements within that package subtree are in
  scope. Absent ⇒ the **whole model**.
- `types` — an optional list of element types; only elements of those types are in scope.
- `status` — an optional list of lifecycle statuses; only elements carrying one of them are
  in scope.
- `tags` — an optional list; only elements carrying at least one of the tags are in scope.

The selectors compose as a logical **AND**. `Baseline` elements are always excluded from the
resolved set (REQ-TRS-BL-002). Scope resolution shall be deterministic and its resolved
membership recorded in the manifest (REQ-TRS-BL-004), so an assessor can see exactly which
elements the seal covers.

- If the resolved scope is **empty** (the selectors match no element), `baseline create`
  shall refuse and write nothing, rather than seal an empty set.

Configuration-projected scope (`--config`) and trace-closure scope (`closureFrom`) are
explicitly **out of scope for v1** and reserved for a later phase; the `frozenScope` schema
shall be forward-compatible with adding them without changing the seal format.
