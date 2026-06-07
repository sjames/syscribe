---
id: REQ-TRS-DISC-003
type: Requirement
name: matrix --features (Feature × Configuration Grid)
title: Tool shall extend matrix with a --features flag printing a Feature × Configuration grid
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** extend the existing `matrix` command with a `--features` flag that prints a **Feature × Configuration** grid: rows are `FeatureDef` elements, columns are `Configuration` elements, and each cell shows whether the feature is selected in that configuration (selected vs not). This complements — and does **not** replace — the default `matrix`, which remains the Requirement × Configuration coverage grid.

The command **shall**:

- support a `--json` flag emitting the grid as structured data (rows, columns, and per-cell selected/not-selected state);
- fall back gracefully and be **dormant** when no feature model is present — if the model contains no `FeatureDef`, `matrix --features` **shall** print a clear "no feature model present" notice and exit `0` rather than erroring;
- leave the default (flag-less) `matrix` behaviour unchanged.

The flag **shall** be discoverable in the `matrix` command's `--help`.

## Rationale

The default `matrix` answers "which configurations cover which requirements". Product-line review also needs the upstream question: "which configurations select which features". A Feature × Configuration grid makes the selection map across all configurations reviewable at a glance — exposing features selected everywhere, nowhere, or in surprising combinations — without reading each `Configuration` file individually.

**Source:** §9 (PLE); product-line feature discoverability; extends the `matrix` command ([[REQ-TRS-VAR-005]]).

**Acceptance criteria:** `matrix --features` prints a grid with one row per `FeatureDef` and one column per `Configuration`, each cell indicating selected vs not selected; `--json` emits the same grid as structured data; on a model with no `FeatureDef` it prints a "no feature model present" notice and exits `0`; the default `matrix` (without `--features`) is unchanged; the flag is listed in `matrix --help`.
