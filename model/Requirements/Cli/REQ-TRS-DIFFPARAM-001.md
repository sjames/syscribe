---
type: Requirement
id: REQ-TRS-DIFFPARAM-001
name: "diff --config compares effective parameter values, not only feature selection"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe diff --config A --config B` and the MCP tool `diff_configs` shall report the parameters whose effective value differs between two Configurations, so two variants that select the same features but bind different values no longer report no difference (GH #466).

- Per parameter (`<FeatureDef>.<param>`): the effective value in A and in B, where each came from (`own`, `inherited`, `default`, `fixed`, `derived`, `bindTo`, as `timeline` reports it), the stage that bound it, and the declared unit. A variant with no value shows it as unset (`null` in JSON).
- Only parameters that differ are listed, in key order; `--all` lists every parameter and marks which differ.
- `--json` carries the rows as `parameters` (always present, empty when none differ) under the published `diff` schema, an additive property.
