---
id: REQ-TRS-SCRIPT-003
type: Requirement
title: Tool shall expose a read-only model API to extension scripts, including custom fields
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide extension scripts ([[REQ-TRS-SCRIPT-001]]) a **read-only model
API** rich enough to inspect the whole model — elements, relationships, computed indices, and
**custom fields**.

### Model access

- A **`model`** value **shall** offer: `model.elements()` (all elements), `model.elements_of_type("<Type>")`
  (filtered by element type), and `model.find("<id-or-qname>")` (one element or unit `()`).

### Element accessors

- Each element **shall** expose: `qname`, `id`, `name`, `title`, `type`, `status`, `doc`
  (the Markdown body), `tags`, and the structural reference lists it declares (`supertype`,
  `typedBy`, `subsets`, `satisfies`, `verifies`, `derivedFrom`, `allocatedTo`, …).
- **`e.field("<key>")`** **shall** return any frontmatter field's value (or unit `()` when
  absent), and **`e.custom_fields`** **shall** return a **map** of the element's
  `custom_fields:` (so scripts can read user/overlay metadata, including `mg_*`).
- The **computed reverse indices** (`verifiedBy`, `derivedChildren`, `refinedBy`,
  `allocatedFrom`, …) **shall** be available on the element.
- Frontmatter values **shall** map to Rhai types: strings → string, lists → array, mappings →
  map, scalars → their Rhai scalar, an absent field → unit `()`.

### Emitting findings

- A **`finding(element, code, severity, message)`** function **shall** record a finding
  attributed to that element's file, for use by validation-hook scripts
  ([[REQ-TRS-SCRIPT-006]]); `severity` is one of `error | warning | info`.

### Output to stdout / stderr

- A **`print(<value>)`** function **shall** write a line to **stdout**, and an
  **`eprint(<value>)`** function **shall** write a line to **stderr**. Both **shall** be
  available to command **and** check functions, for output, progress, and diagnostics while a
  script runs. Text output carries no read capability and is **deterministic**
  ([[REQ-TRS-SCRIPT-002]]) — it is the **only** permitted side effect (still no files,
  network, clock, or randomness).

**Source:** user request — scripts read the model programmatically, including custom fields.

**Acceptance criteria:**

- A script can iterate `model.elements_of_type("Requirement")`, read `e.status`, `e.id`,
  `e.doc`, and `e.field("reqDomain")`.
- `e.custom_fields["supplier"]` returns the value of a `custom_fields: { supplier: … }`
  entry; an absent custom field / `e.field("nope")` returns unit `()`.
- `model.find("REQ-…")` and `model.find("Pkg::Sub::Name")` both resolve the same element;
  an unknown reference returns unit.
- A computed index (e.g. `e.verified_by`) returns the expected reverse links.
- A script calling `print("hi")` writes `hi` to stdout and `eprint("warn")` writes `warn`
  to stderr; both work from a command and from a check.
