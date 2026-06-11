---
id: REQ-TRS-FMA-002
type: Requirement
title: Tool shall expose solver-backed analysis behind feature-check --deep with a defined output and exit contract
status: draft
reqDomain: software
verificationMethod: test
---

The solver-backed analyses ([[REQ-TRS-FMA-003]], [[REQ-TRS-FMA-004]], [[REQ-TRS-FMA-005]]) **shall** be exposed as an opt-in mode of the existing `feature-check` command, invoked as `feature-check --deep`.

### Behaviour

- **Opt-in.** Without `--deep`, `feature-check` behaves exactly as today (extensional checks only — [[REQ-TRS-FM-001]]); the solver engine **shall not** be invoked and **shall not** affect output or exit code.
- **Additive.** `--deep` runs the extensional checks **and** the deep analyses; findings from both are merged into one report.
- **Dormant.** With no `FeatureDef` present, `--deep` **shall** print the "no feature model present" notice and exit `0`, identically to the base command.
- **Exit contract.** Exit `0` when there are no error-severity findings; `1` when there is at least one error-severity finding (`E223` void, `E224` dead feature, `E225` invalid configuration). Warnings (e.g. `W018`) do not fail the command unless explicitly gated.

### Output

- **Text** (default): the merged findings table plus a deep-analysis summary section (void status; counts of dead, core, false-optional features; invalid configurations).
- **`--json`**: a single document carrying a `schemaVersion` and at least:
  - `void` (boolean),
  - `deadFeatures` (list of qualified names),
  - `coreFeatures` (list of qualified names),
  - `falseOptionalFeatures` (list of qualified names),
  - `invalidConfigurations` (list of `Configuration` ids),
  - `findings` (the same `{code, severity, file, message}` items as `validate --json`).
- All lists **shall** be emitted in a deterministic order (qualified-name / id sorted) per [[REQ-TRS-FMA-006]].

The command **shall** be discoverable in `--help`, including the `--deep` flag.

**Source:** ADR-FM-001.

**Acceptance criteria:** `feature-check --deep` is documented in `--help`; on a sound feature model it exits `0`; on a model with an `E223`/`E224`/`E225` finding it exits `1`; with no `FeatureDef` it prints the dormancy notice and exits `0`; `feature-check` *without* `--deep` produces byte-identical output to the pre-feature baseline on the same model; `--json` emits the documented keys with deterministic ordering.
