---
type: Requirement
id: REQ-TRS-LSPFIX-001
name: "LSP quick fixes for the product-line and binding-time diagnostics, validated in memory first"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

`textDocument/codeAction` shall offer a quick fix where one is obvious for each of: `E222` (change to the closest declared parameter or feature), `E754` (remove the rebinding of a value or selection an earlier stage closed), `E751` (use a valid `bindingStage`), `E752` (remove the binding or selection the stage cannot make), `E756` (change the value to the nearest value inside what was narrowed), `W016` (rename `selections:` to `features:`) and `E201` on a `FeatureDef` (add a derived, unused `FEAT-*` id) (GH #421).

- Each fix is a `WorkspaceEdit` and is validated in memory before it is offered, as rename is: the edited file is re-parsed and the whole candidate model re-validated, and the fix is offered only if the diagnostic it answers is gone and it introduces no new error. A fix that would move the problem elsewhere (a different stage that breaks the derivation chain) is therefore not offered. A warning the fix merely reveals does not block it.
- When one fix survives it is preferred. The server never writes the file. A file with none of these findings gets none of these fixes.
