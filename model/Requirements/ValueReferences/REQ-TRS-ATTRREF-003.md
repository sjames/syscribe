---
type: Requirement
id: REQ-TRS-ATTRREF-003
name: "A literal copy of a linked element's attribute value is W095 (opt-in)"
status: draft
reqDomain: software
reqClass: system
tags:
  - references
  - attributes
---

With `[references] copied_values = true` in `.syscribe.toml`, `validate` shall report `W095` when an
element's body (prose or scenario) contains a numeric literal equal to the current value of a named
attribute of an element it links to, without using `{{…}}` for it (GH #452).

1. {#links} **Links.** Every outgoing reference counts: `satisfies`, `verifies`, `verifiesElement`,
   `derivedFrom`, `refines`, `allocatedTo`/`allocatedFrom`, `typedBy`, `supertype`, `achieves`,
   `evidence` `ref:`, user-defined `links:`, and the elements of the body's own `{{…}}` references.
2. {#noise} **Noise filter.** A literal is a whole token (not part of an id, a version, a date or a
   word); thousands separators (space, `,`, `_`, `'`) are read; literals whose value is below 10 and
   integers from 1900 to 2100 (years) are ignored; code is skipped as for references.
3. {#config} **Config.** `copied_values` must be true or false; anything else is reported like the
   table's other problems (`W094` on `.syscribe.toml`). `lint-docs` applies the same check to
   external documents, using the elements they reference with `{{…}}`.
