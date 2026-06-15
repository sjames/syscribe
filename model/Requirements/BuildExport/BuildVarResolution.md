---
type: Requirement
id: REQ-TRS-BUILD-020
name: "Build variable resolution order: buildExports → parameterBindings → buildOverrides"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-010]
breakdownAdr: Decisions::BuildExportADR
tags:
  - resolution
  - build-integration
---

When resolving the build variable set for a configuration, the `build-config`
subcommand shall apply the following three-step procedure in order, with each step
potentially overwriting variables set by earlier steps:

**Step 1 — Feature selection (`buildExports:`)**

For each `FeatureDef` in the model that declares `buildExports:`:

- If the feature is **selected** in the configuration, emit each declared variable
  with its `whenSelected:` value (default `1` when omitted).
- If the feature is **not selected** and the entry declares `whenDeselected:`, emit
  the variable with that value.
- If the feature is **not selected** and `whenDeselected:` is absent, the variable
  is not emitted.

**Step 2 — Parameter bindings (`buildVar:` + `parameterBindings:`)**

For each `FeatureDef` parameter that declares `buildVar:`:

- If the active configuration's `parameterBindings:` supplies a value for this
  parameter, emit `buildVar = <bound value>`.
- Otherwise, if the parameter declares a `default:`, emit `buildVar = <default>`.
- Otherwise, omit the variable.

**Step 3 — Configuration overrides (`buildOverrides:`)**

Apply every entry in the active `Configuration`'s `buildOverrides:` map, overwriting
any variable of the same name established in Steps 1 or 2.

The final set is emitted in variable-name alphabetical order to ensure reproducible,
diff-friendly output regardless of the order elements appear in the model.
