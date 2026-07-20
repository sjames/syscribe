---
type: Requirement
id: REQ-TRS-LSP-004
name: "Find-references returns every element that cross-references the element under the cursor"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
  - navigation
---

The server shall implement `textDocument/references`. When invoked on an element's own
identity (its `id`/`name` frontmatter value, or its qname), the server shall return the
`Location` of every field in every other file that cross-references it — the same set already
computed as reverse indices (`verifiedBy`, `derivedChildren`, `satisfiedBy`, allocation
consumers, and equivalent reverse edges for other reference kinds).

## Behavior

- Results are file+field locations of the *referencing* value (e.g. the `verifies:` entry in
  the verifying `TestCase`'s frontmatter), not the target element's own file.
- If the element under the cursor has no incoming references, the server returns an empty
  result, not an error.
- `includeDeclaration` (per the LSP spec) controls whether the element's own defining location
  is included alongside the reverse-reference locations.
