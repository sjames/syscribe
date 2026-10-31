---
id: REQ-TRS-BT-000
type: Requirement
name: "Binding time is a staged-configuration axis, from declaration through to the running system"
status: draft
reqDomain: software
verificationMethod: test
---

Syscribe shall treat binding time as a first-class, staged-configuration axis of a product line: the stage at which each feature selection and parameter value is resolved shall be declared, ordered, checked, reported when still open, and — for values fixed at load or run time — delivered to the running system as a generated parameter file. Extends [[REQ-TRS-PARAM-004]].

## Rationale

Today `bindingTime:` is a per-parameter label over a fixed triad that only feeds a few checks. It cannot say who binds a value or what remains open, it does not apply to features, and a system with `load`/`runtime` parameters has no artefact to read them from (decision: `ADR-SYS-BT-001`).

## Scope

Stage vocabulary; binding windows on features and parameters; `bindingStage:` on `Configuration` over the `derivedFrom:` chain; residual variability and partial-binding constraint evaluation; the YAML values file, its JSON Schema and checker; provenance; projection; opt-in verification rules; the feature-model UI; additive migration. Out of scope: per-consumer parameter files and binder roles tied to the `[users]` roster.
