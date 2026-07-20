---
type: Requirement
id: REQ-TRS-LSP-014
name: "codeAction offers a breakdownAdr quick-fix for E310, one action per accepted ADR"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCodeActionsADR
tags:
  - lsp
  - codeaction
---

The server shall implement `textDocument/codeAction` with `codeActionKinds: ["quickfix"]`.
When the requested range covers an `E310` diagnostic (a `Requirement` with `derivedFrom:` and
no `breakdownAdr:`), the server shall return one `CodeAction` per `accepted` `ADR` currently in
the model, each titled with that ADR's name and each a direct `WorkspaceEdit` inserting
`breakdownAdr: <that ADR's qname>` into the requirement's frontmatter.

## Behavior

- If the model has zero `accepted` ADRs, no `E310` quick-fix action is offered.
- The insertion is a single-line `TextEdit` placed immediately before the frontmatter's closing
  `---`; it does not rewrite the rest of the file.
- No action is offered for a range that does not cover an `E310` diagnostic.
