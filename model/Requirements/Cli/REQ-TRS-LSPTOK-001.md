---
type: Requirement
id: REQ-TRS-LSPTOK-001
name: "LSP semantic tokens for feature, parameter and stage references"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

The language server shall advertise and answer `textDocument/semanticTokens/full` for the variability fields of a model file's frontmatter (GH #422): feature qualified names (`class`) and `FEAT-*` ids (`enumMember`), parameter paths `<feature>.<parameter>` (`property`) and binding-time stage names (`keyword`), in `appliesWhen:`, a Configuration's `features:` keys, `requires:`/`excludes:`, `parameterBindings:`/`parameterNarrowing:` keys, a constraint's `expression:` and `bindingTime:`/`latestBindingTime:`/`bindingStage:` values.

- A reference that names no feature, parameter or stage of the model carries the `unresolved` modifier, so a typo is visible before validation runs.
- Operators, numbers, values, other fields and the Markdown body are not tokens. The legend (four types, one modifier) is documented in `syscribe help lsp`.
