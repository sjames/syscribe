---
type: Requirement
id: REQ-TRS-ATTRREF-005
name: "refs Elem.attr lists the uses, literal copies and stale copies of a value"
status: draft
reqDomain: software
reqClass: system
tags:
  - references
  - attributes
  - cli
---

`syscribe refs <Elem.attr>` shall print the attribute's current value and list, in separate
sections, every `{{Elem.attr}}` use, every unreferenced literal copy of the current value, and every
stale copy of a previous value in the elements that link to the attribute's element (GH #452).
The element part resolves as for value references; when it does not resolve, or is ambiguous, the
command says so and exits 1 instead of searching elements by name. `--json` prints the same as one object `{attribute, element, path, value, unit, uses, copies, stale}`.
