---
type: ADR
id: ADR-SYS-BUILD-001
name: "Co-locate build variable mappings with FeatureDef elements; Configuration carries overrides only"
status: accepted
tags:
  - build-integration
  - variability
  - feature-model
---

## Context

The feature model captures which capabilities are selected in a given `Configuration`,
and `parameterBindings` records the numeric/enum values of each feature's typed
parameters. Build systems (CMake, Make, Kconfig, C preprocessor) need these selections
expressed as named variables — `ENABLE_ABS=1`, `ABS_THRESHOLD_MS=40` — without manual
translation.

Three placement options were considered for the feature-to-variable mapping:

1. **On the `FeatureDef`** — each feature declares the variable(s) it drives when selected.
2. **On the `Configuration`** — each product configuration explicitly lists every build variable.
3. **In a separate mapping file** — a standalone YAML that cross-references features to variables.

## Decision

Option 1: build variable mappings (`buildExports:` and `buildVar:` on parameters) live on the
`FeatureDef`. A `Configuration` may add a `buildOverrides:` map for variables that are
configuration-specific rather than feature-derived (e.g. version strings, product SKU names).

All three new fields are **optional**. A model that carries no build annotations remains
fully valid; the `build-config` command is an opt-in capability.

## Rationale

Co-location keeps the mapping close to the feature definition, so when a feature is
renamed or its parameter range changes the author is already looking at the right file.
Centralising every variable in `Configuration` would require editing every product
definition whenever a feature gains a new build variable. A separate mapping file adds
indirection for little gain at this scale.

`buildOverrides:` on `Configuration` handles the residual case: variables that differ
per product for reasons outside the feature tree (branding, version strings, OEM SKUs).

## Consequences

- `build-config` resolves variables in a deterministic three-step order:
  feature `buildExports:` → `parameterBindings` → `buildOverrides:` (last writer wins).
- Variable name collisions across two selected features (before overrides) are
  validation error **E050**; the model is ill-formed and `build-config` aborts.
- `W050` (opt-in) warns when a selected feature contributes no build variable, so
  models that intend full coverage can gate on it in CI.
