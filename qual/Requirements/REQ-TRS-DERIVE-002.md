---
id: REQ-TRS-DERIVE-002
type: Requirement
name: "Derive engine shall support aggregate operators: sum, max, min, count, collect"
status: draft
reqDomain: software
verificationMethod: test
---

The derive expression language **shall** support the following aggregate operators applied over a collection source:

- **`sum(source.field)`** — numeric sum; absent fields coerce to 0 (W500 opt-in).
- **`max(source.field)`** — numeric maximum; null if collection is empty.
- **`min(source.field)`** — numeric minimum; null if collection is empty.
- **`count(source)`** — number of elements in the collection (after filtering).
- **`collect(source.field)`** — list of all field values (strings or numbers).

Collection sources: `children` (direct directory children), `parent` (single parent).

**Acceptance criteria:**

- `sum(children.custom_fields.wcet)` returns the sum of all children's `wcet` custom field.
- `count(children)` returns the number of direct children.
- `max(children.silLevel)` returns the highest SIL level among children.
