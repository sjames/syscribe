---
type: ADR
id: ADR-SYS-BT-001
name: "Binding time as a staged-configuration axis: extensible stages, binding windows, bindingStage on Configuration, generated late-bound parameter file"
status: accepted
tags:
  - variability
  - binding-time
---

## Context

`bindingTime:` (`REQ-TRS-PARAM-004`) is a per-parameter label over a fixed `compile < load < runtime` triad. It feeds `E229`/`E230`/`W027` and suppresses `W017`/`W248`, but nothing says *who* binds a value, *when*, or what remains open afterwards; features cannot carry a binding time at all; `load` has no behaviour of its own; and the rank logic is duplicated across `feature_model.rs`, `validator.rs` and `placeholders.rs` with magic numbers. A system with `load`/`runtime` parameters also has no generated artefact to read them from.

## Decision

1. **Staged configuration.** Binding time is a lifecycle *stage* axis over the whole product line (Czarnecki-style staged configuration). A `Configuration` declares `bindingStage:`; later stages specialise earlier ones through the existing `derivedFrom:` Configuration inheritance (§9.8); what remains open is *residual variability*.
2. **Extensible vocabulary.** Stages are an ordered vocabulary, default `compile < load < runtime`, refinable by `[bindingTimes.<name>]` in `.syscribe.toml` (the `[linkTypes]` pattern). *Rejected:* a fixed triad — it cannot model real build/deploy pipelines.
3. **Windows, on features and parameters.** `bindingTime:` is the earliest stage and `latestBindingTime:` the latest; a single value is the degenerate window; absent means unspecified and opts out of every check. Features gain it too. *Rejected:* parameters only — a feature's selection is the usual PLE binding-time question.
4. **`bindingStage:` on `Configuration` plus a `derivedFrom:` chain**, with closed ancestor values and narrow-only ranges. *Rejected:* one Configuration with a stage per binding — less reuse, no monotonic chain to check.
5. **Stage violations are errors for staged Configurations**; `W027` stays for unstaged ones, so existing models are unchanged.
6. **Generated late-bound parameter file.** `params export` writes a deterministic YAML values file keyed by stable `FEAT-*` id, from a late-stage Configuration; a JSON Schema and `params check` validate it; the model never ingests it. Strict by default (`--allow-open` overrides). *Rejected:* keying by qname (breaks on rename); reading the file back into the model (two sources of truth).
7. **One `BindingTime` type** replaces every literal comparison.
8. **Everything additive.** A model using none of the new fields, with no `[bindingTimes]`, behaves exactly as today.

## Rationale

The existing pieces (`derivedFrom:` inheritance, `parameterBindings:`, the SAT engine's forced/free analysis, `W090`/`W091`-style hash staleness, the `[linkTypes]` config pattern) already cover most of the machinery; the new surface is the stage axis, the window check and the export.

## Consequences

- Phased delivery: vocabulary and windows (with the single `BindingTime` type), then stages and provenance, then residual variability and the parameter file, then projection, verification and UI.
- Projection retaining elements bound after the projected stage is the riskiest change, because it alters projection output; it is its own phase.
- New diagnostic codes are allocated at implementation and registered in `prompts/spec/validation.md` and `docs/validation/rules.md`.
- Out of scope: per-consumer split of the parameter file, binder roles tied to the `[users]` roster.
