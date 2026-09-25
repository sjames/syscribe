---
id: REQ-TRS-DERIVE-001
type: Requirement
name: "Element frontmatter shall support a derive: block for computed field values"
status: draft
reqDomain: software
verificationMethod: test
---

Any element's YAML frontmatter **shall** accept an optional **`derive:`** mapping whose keys are field names to compute and whose values are formula strings.

```yaml
derive:
  wcetConsumed: sum(children.custom_fields.wcet)
  wcetHeadroom: self.custom_fields.wcetBudget - self.wcetConsumed
```

Derived fields are evaluated in dependency order: a field is evaluated after every derived field it references (`self.<fieldName>`, `elements["QName"].<fieldName>`, or a `children`/`parent` aggregate over `<fieldName>`), so later entries may reference earlier ones via `self.<fieldName>` and a cross-element chain evaluates regardless of file order. Independent fields keep top-to-bottom, file-walk order. Derived values are stored on the element as computed fields visible to subsequent passes (validation, query). A cyclic dependency is [[REQ-TRS-DERIVE-004]].

`derive` **shall** be a recognised top-level frontmatter field on every element type: declaring a `derive:` block **shall not** raise the unrecognised-field warning `W047` (GH #141 — it previously did, because the block was read out of the `extra` catch-all). A `derive:` value that is not a mapping, or a formula value that is not a string, **shall** be reported as `E505` rather than silently ignored.

**Acceptance criteria:**

- An element with a valid `derive:` block evaluates without error.
- `query show <qname>` includes derived fields in the output.
- Derived values produced by top entries are visible to bottom entries via `self.<field>`.
- An element with a `derive:` block raises no `W047` for it.
- A derived field referencing another element's derived field evaluates to the computed value even when that element is walked later.
- `derive: 5` (not a mapping) and a non-string formula each raise `E505`.
