---
id: REQ-TRS-VAR-007
type: Requirement
name: A Configuration shall inherit feature selections and parameter bindings from one base Configuration through derivedFrom
status: draft
reqDomain: software
verificationMethod: test
---

A `Configuration` **shall** accept an optional `derivedFrom:` naming **one** base `Configuration` of
the same model, by qualified name or `CONF-*` id (spec §9.8), and the tool **shall** treat the
child's selection as the **effective** selection defined below.

### Effective selection

- **Features.** The effective `features:` selection is the base's effective selection overlaid by
  the child's own `features:` entries; a child entry replaces the inherited entry for the same
  feature (a `FEAT-*` id and the FeatureDef's qualified name denote the same feature). Chains
  compose: a configuration derived from a derived configuration inherits through both.
- **Parameter bindings.** Every `parameterBindings:` entry the child declares is kept; an inherited
  entry is kept unless the child binds the same `<feature>.<param>` itself or the child's own
  `features:` sets that feature to `false`.
- **Nothing else** (`status`, `featureModel`, `subConfigurations:`, `buildOverrides:`, …) is
  inherited.
- The authored file **shall not** be rewritten; inheritance is a computed view.

### Consumers

The effective selection **shall** be used by the `--config` projection lens, `matrix`, `configure`,
`validate --config`/`--all-configs`, `feature-check`, `build-config`, the per-Configuration
validation rules (parameter binding, group, coverage and missing-selection rules), and
`subConfigurations:` consolidation (a consolidated Configuration's inherited bindings close its
parameters). `show` **shall** mark entries the file inherits.

### Checks

A Configuration's `derivedFrom:` is not a requirement derivation and **shall not** raise `E105`,
`E017` or contribute to `derivedChildren`. Instead the tool **shall** report:

| Code | Condition |
|---|---|
| `E234` | the base does not resolve to any element of the model |
| `E235` | the base resolves to an element that is not a `Configuration` |
| `E236` | the configuration is on a `derivedFrom:` inheritance cycle (reported on each member) |
| `E237` | more than one base is named |
| `E215` | the base is not `approved` or `released` |

A configuration with any of `E234`–`E237` inherits nothing.

**Source:** spec §9.8, §9.11, §9.12; GH #137.

**Acceptance criteria:** a child naming an approved base validates without `E105`, `W016` or `W017`
and projects (`list --config`) onto the union of inherited and own selections; a child that
deselects a feature drops the inherited binding for it (no `E203`) and projects without it; a
child's own binding replaces the inherited value (an out-of-range override raises `E205` on the
child only); `configure` and `matrix` use the effective selection; `validate --all-configs` passes;
a consolidating tier whose `subConfigurations:` names an inheriting peer Configuration raises
neither `E518` nor `W513` for the inherited binding; each of `E234`, `E235`, `E236`, `E237` and
`E215` is raised for its crafted condition and none of them raises `E105` or `E017`.
