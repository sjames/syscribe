---
type: Requirement
id: REQ-TRS-LSPHOVER-001
name: "LSP hover and code lenses show where an element is active and where a parameter is bound"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

The language server shall show variability in hover and code lenses (GH #420):

- hovering an element's `appliesWhen:` key lists the Configurations it is active in and those it is not; hovering a key of a Configuration's `parameterBindings:` or `parameterNarrowing:` shows the parameter's type, unit, range or enumeration, binding window and default, the stage it is bound or narrowed at, and the parameter constraints it takes part in with their state for that Configuration;
- a code lens above a gated element says `active in N of M configurations`; a code lens on each parameter of a `FeatureDef` says `bound at <stage> by <Configuration>` (with the number of others), or `unbound` with its default;
- the values come from the projection, provenance and constraint code the CLI uses. Hovering a feature name inside `appliesWhen:` still shows that feature, and the earlier lenses are kept. Nothing is written.
