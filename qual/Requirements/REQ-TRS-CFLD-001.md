---
id: REQ-TRS-CFLD-001
type: Requirement
title: Tool shall recognise a custom_fields map on any element and shape-check its values
status: draft
reqDomain: software
verificationMethod: test
---

Users **shall** be able to attach arbitrary user-defined data to **any** element via a
dedicated `custom_fields:` frontmatter map, making custom data **intentional and
addressable** rather than silently swallowed by the unknown-key catch-all.

### Schema

- `custom_fields:` is an optional **flat map** of `string -> scalar | list-of-scalars`
  (e.g. `supplier: Bosch`, `partNumbers: [A-1001, A-1002]`). It **shall** be accepted on
  every element type.
- Keys are **freeform** — any key is allowed; no registration or name validation. This
  fits the LLM-authoring workflow.
- The map **shall** serialise in a stable (sorted) order so writes do not produce noisy
  round-trip diffs.
- The pre-existing unknown-top-level-key catch-all is unchanged; `custom_fields:` is the
  **intentional** home for custom data, distinct from accidental unknown keys.

### Shape validation

- The validator **shall** check only the **shape** of each `custom_fields` value: a
  value that is **not** a scalar or a list of scalars (e.g. a nested map, or a list
  containing a map) **shall** raise warning `W041` (`custom field '<key>' must be a
  scalar or a list of scalars`). No other validation of custom data is performed.
- `W041` is a **warning** (custom data is advisory, never a hard error); it is
  CI-gateable via `--deny W041`.

### Dormancy

- An element with no `custom_fields:` is unaffected; the feature adds no findings to
  models that do not use it.

**Source:** GH #39 (custom fields). Note: the issue text suggested code `W024`, which is
already in use (orphan-feature warning); this requirement uses the next free code
`W041`.

**Acceptance criteria:**

- An element declaring `custom_fields:` with scalar and list-of-scalar values parses
  cleanly (no `W041`), and the values round-trip in sorted order.
- A `custom_fields` value that is a nested map (or a list containing a map) raises
  `W041` naming the offending key.
- A model with no `custom_fields:` produces no `W041` and is otherwise unchanged.
