---
id: REQ-TRS-FM-001
type: Requirement
title: Tool shall provide a feature-check command for holistic feature-model validation, separate from validate
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a dedicated `feature-check` command that performs holistic, feature-model-wide validation (cross-tree constraints, group consistency, parameter integrity). These analyses are **deliberately not** run by the default `validate` command — `validate` keeps its existing per-element behaviour unchanged.

The command **shall**:

- report findings with the same `| code | file | message |` shape as `validate`, drawing on the structural rules ([[REQ-TRS-FM-002]]) and parameter-integrity rules ([[REQ-TRS-FM-003]]);
- use the exit-code contract: `0` when there are no error-severity findings, `1` when there is at least one error-severity finding;
- support a `--json` flag emitting the findings as structured data;
- be **dormant** when no feature model is present — if the model contains no `FeatureDef`, it **shall** print a clear "no feature model present" notice and exit `0` rather than erroring.

The command **shall** be discoverable in `--help`.

**Source:** §9 (PLE); follow-up to make deep feature-model validation explicit and opt-in.

**Acceptance criteria:** `feature-check` is listed in `--help`; on a model with a feature model and a structural or parameter violation it prints the finding(s) and exits `1`; on a clean feature model it exits `0`; on a model with no `FeatureDef` it prints a "no feature model present" notice and exits `0`; `--json` emits structured findings. The default `validate` command's output is unchanged by the existence of this command.
