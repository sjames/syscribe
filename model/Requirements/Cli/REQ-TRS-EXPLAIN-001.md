---
type: Requirement
id: REQ-TRS-EXPLAIN-001
name: "explain-feature says why a feature is forced or excluded and what would select it"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe explain-feature <feature> [--from <Configuration>] [--select <feature>=<bool>]... [--json]`, the MCP tool `explain_feature` and the LSP request `syscribe/explainFeature` shall report a feature's state for a partial selection (selected, deselected, forced on, forced off, free, or conflict) and why (GH #417).

- The cause is the smallest set of the user's choices and of the model's constraints; each constraint names its kind (requires, excludes, mandatory, group at-least / at-most, child-implies-parent, root), label, declaring feature and file, so a front end can link to it.
- `toSelect` lists the choices any one of which can be dropped to make the feature selectable, or says the model alone prevents it. A feature reported dead by `feature-check --deep` is reported model-level.
- With a Configuration, the parameter constraints that mention the feature's parameters are listed with their state. Nothing is written; the payload has the published `explain-feature` schema.
