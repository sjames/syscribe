---
id: REQ-TRS-DERIVE-003
type: Requirement
name: "Derive engine shall support cross-element address and arithmetic"
status: draft
reqDomain: software
verificationMethod: test
---

The derive expression language **shall** support:

- **`self.<field>`** — the current element's own field (standard or custom_fields).
- **`self.custom_fields.<key>`** — access a custom field by key.
- **`elements["Qualified::Name"].<field>`** — address a specific element by qualified name.
- **Arithmetic**: `+`, `-`, `*`, `/` between numeric sub-expressions.
- **Null coalesce**: `expr ?? defaultValue` — substitutes `defaultValue` when `expr` is null/absent.

**Acceptance criteria:**

- `self.custom_fields.budget - self.custom_fields.used` evaluates correctly.
- `elements["Sys::Budget"].custom_fields.total` fetches a field from a named element.
- `self.custom_fields.x ?? 0` returns 0 when `x` is absent.
