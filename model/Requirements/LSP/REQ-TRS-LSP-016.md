---
type: Requirement
id: REQ-TRS-LSP-016
name: "codeAction offers no quick-fix for diagnostics other than E310 and W090"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCodeActionsADR
tags:
  - lsp
  - codeaction
---

`textDocument/codeAction` shall return no `CodeAction` for a requested range whose diagnostics
are exclusively codes other than `E310` or `W090` — v3 deliberately does not attempt a
general-purpose quick-fix framework (`ADR-SYS-LSP-003`). A range covering a mix of `E310`/`W090`
and other codes still gets the `E310`/`W090` actions; the other codes simply contribute none.
