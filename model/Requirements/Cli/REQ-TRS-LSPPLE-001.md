---
type: Requirement
id: REQ-TRS-LSPPLE-001
name: "LSP completion is aware of features, parameters, enumerations and binding stages"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

`textDocument/completion` shall offer, in context, feature qualified names and `FEAT-*` ids (in `appliesWhen:`, a Configuration's `features:` keys, `requires:` and `excludes:`), parameter paths (in `parameterBindings:` keys and `parameterConstraints:` expressions), the `enumValues:` of the parameter being bound (or `true`/`false` for a boolean), and the stage names of the project's binding-time vocabulary (for `bindingTime:`, `latestBindingTime:` and `bindingStage:`) (GH #419).

- Items carry documentation: for a parameter its type, unit, range, enumeration, default, requiredness and binding window; for a feature its id, group, parameters and window. Each replaces the whole token under the cursor, a qualified name included, but not the colon that ends a key.
- Only these fields are claimed; completion elsewhere (cross-references, `status`, `type`) is unchanged. The server never writes files.
