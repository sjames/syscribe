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

Derived fields are evaluated in top-to-bottom order within a single element so that later entries may reference earlier ones via `self.<fieldName>`. Derived values are stored on the element as computed fields visible to subsequent passes (validation, query).

**Acceptance criteria:**

- An element with a valid `derive:` block evaluates without error.
- `query show <qname>` includes derived fields in the output.
- Derived values produced by top entries are visible to bottom entries via `self.<field>`.
