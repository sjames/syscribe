---
id: REQ-TRS-PROJ-004
type: Requirement
title: feature-check --deep shall prove no reference escapes in any valid configuration
status: draft
reqDomain: software
verificationMethod: test
---

The per-configuration lens ([[REQ-TRS-PROJ-003]]) only finds escapes in the variant being viewed. `feature-check --deep` **shall** additionally **prove**, across *all* valid configurations, that a referenced element is active wherever its referrer is — catching latent escapes in configurations nobody has authored.

### Rule

For each reference edge `X → Y`, the implication `appliesWhen(X) ⇒ appliesWhen(Y)` **shall** hold in every valid configuration of the feature model. Operationally, using the SAT encoding (`Φ` = the feature-model constraints, [[REQ-TRS-FMA-001]]):

> if `Φ ∧ appliesWhen(X) ∧ ¬appliesWhen(Y)` is **satisfiable**, the implication is violable — a valid configuration activates `X` without `Y`.

| Edge class (per [[REQ-TRS-PROJ-003]] taxonomy) | Finding when violable |
|---|---|
| Structural / typing | `E227` — **error** |
| Traceability | `W020` — **warning** |

- An element with no `appliesWhen:` has `appliesWhen = true`; thus an always-active `X` referencing a `Y` that can be deselected is correctly flagged (a latent escape).
- Each finding **shall** include a **witness**: a feature selection (from the satisfying assignment) that demonstrates the violation, so it is actionable.
- The check is config-independent, deterministic, Boolean-layer-only, and dormant when no feature model is present.

**Source:** ADR-PROJ-001; ADR-FM-002 (SAT engine).

**Acceptance criteria:** A structural edge from an always-active `X` to a `Y` with `appliesWhen: Features::F` yields `E227` with a witness selection in which `F` is deselected; when `appliesWhen(X) ⇒ appliesWhen(Y)` holds (e.g. `X` is `appliesWhen "Features::F and Features::G"`, `Y` is `appliesWhen Features::F`) no finding is produced; the traceability analog yields `W020`; results are identical across runs.
