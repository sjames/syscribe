---
type: Requirement
id: REQ-TRS-LSP-010
name: "prepareRename accepts a stable-id position and rejects everything else"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCompletionRenameADR
tags:
  - lsp
  - rename
---

The server shall implement `textDocument/prepareRename`. Given a cursor position, it shall
resolve the token exactly as `textDocument/hover`/`definition` do (`REQ-TRS-LSP-003`), and:

- If the token resolves to a **stable-id-identified** element (`Requirement`, `TestCase`,
  `ADR`, `FeatureDef`, `Baseline`, or a type with a configured `[ids.prefixes]` entry), return
  the range of the id token and the id itself as the placeholder text.
- If the token resolves to a **name-identified** (qname-only) element, or does not resolve at
  all, return an LSP error explaining that only stable-id renames are supported — a
  name/qname rename is a file move (`syscribe mcp move_element`), out of scope for
  `textDocument/rename`.
