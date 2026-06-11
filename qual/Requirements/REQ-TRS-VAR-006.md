---
id: REQ-TRS-VAR-006
type: Requirement
title: Tool shall apply a Package's appliesWhen transitively to its whole subtree, with one declaration per path
status: draft
reqDomain: software
verificationMethod: test
---

To enable or disable a cohesive group of model elements together (e.g. a whole variant subtree of requirements, architecture, and tests), a `Package` (a namespace `_index.md`) **shall** be permitted to declare an `appliesWhen:` field. The package's condition **shall** apply **transitively** to every element contained in that package's subtree — directly and through nested sub-packages — using the same boolean grammar and `FeatureDef` operands as element-level `appliesWhen:` (§9; every operand must resolve to a `FeatureDef`, else `E209`).

## Effective condition

Every element has a single **effective condition** that the variability dimension evaluates:

- the element's **own** `appliesWhen:` if it declares one; otherwise
- the `appliesWhen:` of the **nearest ancestor package** that declares one; otherwise
- **always active** (no condition).

Effective conditions are never combined: an element's effective condition is exactly one declaration (its own, or one ancestor package's), or none. Every consumer of activation — the `--config` projection lens, escaping-reference checks (`E226`/`W019`), `matrix` (incl. `matrix --features`), `why-active`, the `feature <qname>` gates list, `list --feature`, and `feature-check --deep` reference-edge proofs (`E227`/`W020`) — **shall** use the effective condition rather than only the element's own `appliesWhen:`.

A feature referenced **only** by a package's `appliesWhen:` (gating contents but never an element directly) **shall** count as referenced for the orphan-feature rule (`W024`).

## The one-declaration-per-path invariant (E228)

To keep effective conditions unambiguous and validatable, **at most one** node on any root-to-leaf path may declare `appliesWhen:`. The tool **shall** emit error **`E228`** for an invalid `appliesWhen:` placement:

1. **Nested declaration** — an element or a sub-package `_index.md` declares `appliesWhen:` while some ancestor package in its subtree already declares one. Every offending nested declaration (even one identical to the ancestor's) is an error; the message names both the offending node and the owning ancestor package.
2. **Forbidden target** — `appliesWhen:` would attach to the variability machinery itself, or to the whole model. Because the tool identifies the feature model and configurations by element **type** (there is no dedicated "Features package" or "Configurations package" type — they are ordinary namespaces known only by the `FeatureDef`/`Configuration` elements they contain), this rule is expressed in terms of element type:
   - a `FeatureDef` or a `Configuration` **shall not** carry its own `appliesWhen:`; and
   - a package that declares `appliesWhen:` **shall not** contain any `FeatureDef` or `Configuration` anywhere in its subtree (this is what rejects gating the `Features/` or `Configurations/` namespace); and
   - the **model-root** package `_index.md` **shall not** declare `appliesWhen:` (it would project the entire model to empty).

`E228` is opt-in (only relevant once `appliesWhen:` is used) and dormant when no feature model is present.

## Gates-nothing warning (W026)

The tool **shall** emit warning **`W026`** when a package declares `appliesWhen:` but its subtree contains no projectable element (it gates nothing), mirroring the orphan-feature warning `W024`. Gate with `--deny W026`.

## Backward compatibility

The feature is purely additive: a model with no package-level `appliesWhen:` behaves exactly as before. Element-level `appliesWhen:` is unchanged and remains the mechanism for conditioning individual elements or a strict subset of a package's contents (in which case the package itself must **not** declare `appliesWhen:`).

**Source:** §9 (variability); follow-up to [[REQ-TRS-VAR-001]] and the projection requirements [[REQ-TRS-PROJ-001]].

**Acceptance criteria:**

1. A package declaring `appliesWhen: Features::X` with several contained elements (none carrying their own `appliesWhen:`): under `validate --config` for a configuration **with** `X`, all contained elements are active; for a configuration **without** `X`, none are active (the subtree is filtered out, including the package node), and `list`/`matrix`/`export` reflect the same.
2. An element inside that package that **also** declares `appliesWhen:` produces exactly one `E228` naming the element and the ancestor package; likewise a nested sub-package `_index.md` that declares `appliesWhen:` produces `E228`.
3. `appliesWhen:` produces `E228` when declared on: the model-root `_index.md`; a `FeatureDef`; a `Configuration`; or a package whose subtree contains a `FeatureDef` or `Configuration` (e.g. the `Features/` or `Configurations/` namespace).
4. `why-active <element> --config C` for a transitively-gated element reports the verdict and attributes the condition to the owning package (provenance).
5. A reference from an always-active element **outside** the gated package into an element **inside** it escapes the variant in a configuration where the package condition is false (`E226` structural / `W019` traceability), while references **between two elements inside** the same gated subtree never escape.
6. A package declaring `appliesWhen:` with an empty subtree produces `W026`; `--deny W026` exits non-zero.
7. With no package-level `appliesWhen:` anywhere, behaviour is identical to the prior release (regression).
