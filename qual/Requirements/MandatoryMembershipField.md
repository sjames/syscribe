---
id: REQ-TRS-FM-004
type: Requirement
title: Tool shall accept an optional boolean mandatory field on a FeatureDef, orthogonal to groupKind child grouping
status: draft
reqDomain: software
verificationMethod: test
---

The feature model's `groupKind:` field currently conflates two orthogonal concepts: a feature's **membership** relationship to its parent (mandatory vs optional) and the **grouping** of that feature's children (`and`/`or`/`alternative` plus cardinality). Because there is only one `groupKind` per feature, a node cannot be both *mandatory under its parent* and *an XOR/alternative group over its children*. This requirement introduces an additive, backward-compatible separation of the two concerns (see [[ADR-FM-003]]).

The tool **shall**:

- accept an **optional boolean `mandatory:`** field on a `FeatureDef`, declaring that feature's membership relationship to its parent, **independently** of `groupKind:`;
- interpret `mandatory: true` on a feature **with a parent** as *selected whenever the parent is selected* (parent ⇒ feature), and on a **top-level** feature (no parent) as *always selected* (root-mandatory);
- treat `mandatory: false` **or absent** as **optional** membership — the default;
- continue to use `groupKind:` (`alternative` / `or` / `and`, with `cardinality`) **only** to govern the grouping of a feature's **children**, so a single `FeatureDef` may be **both** `mandatory: true` **and** `groupKind: alternative` — a *mandatory XOR group* in which exactly one child is selected in every product;
- preserve backward compatibility: the legacy `groupKind: mandatory` form **shall** continue to mean a mandatory member (equivalent to `mandatory: true`) for any feature that does not set `mandatory:`;
- reflect the field correctly in solver-backed analysis (`feature-check --deep`, [[REQ-TRS-FMA-001]]): a `mandatory: true` group node becomes a **core feature**, its `alternative` grouping still forces **exactly one** child, and the field by itself introduces **no** void or dead features.

The tool **should** (not shall) warn that using `groupKind: mandatory` on a node that **has children** is ambiguous and recommend `mandatory: true` together with an explicit child `groupKind:`. This requirement does **not** mandate a new validation code for that advisory.

**Source:** §9 (PLE); feature-model schema separation of membership from child grouping ([[ADR-FM-003]]).

**Acceptance criteria:**

- A `FeatureDef` with `mandatory: true` + `groupKind: alternative` and child features yields: the parent is reported as a **core feature**; **exactly one** child is selected in every valid configuration; `feature-check --deep` reports the model **sound** (not void, no dead features).
- The bundled `model/` UAV product line can **drop** its synthetic `Features::Base` feature by instead marking `Features::Propulsion`, `Features::Payload`, and `Features::DataLink` as `mandatory: true`, and still pass `feature-check --deep` with all `CONF-UAV-*` configurations valid.
- A legacy feature using `groupKind: mandatory` (a leaf, with no `mandatory:` field) is still **forced selected** (regression guard).
