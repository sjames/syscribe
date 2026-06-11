---
type: ADR
id: ADR-FM-003
title: "Separate feature membership (mandatory:) from child grouping (groupKind:)"
status: accepted
---

## Context

The feature model's `groupKind:` field conflates two orthogonal concepts:

- **membership** — how a `FeatureDef` relates to its **parent** (mandatory vs optional), and
- **grouping** — how a `FeatureDef`'s **children** are organised (`and` / `or` / `alternative`, with `cardinality`).

Because there is only **one** `groupKind` per feature, a node cannot be **both** "mandatory under its parent" **and** "an XOR/alternative group over its children". A feature that should be a *mandatory XOR group* (always present, exactly one child) is inexpressible directly.

The bundled UAV model demonstrates the cost of this conflation. `Features::Propulsion`, `Features::Payload`, and `Features::DataLink` are each `groupKind: alternative` (XOR groups), but each must **also** ship in every product. To force their presence we introduced a **synthetic `Features::Base`** feature carrying `groupKind: mandatory` and a `requires:` list pointing at all three groups. That workaround inflates the feature model with a node that exists only to express membership the schema could not, and couples the three groups' mandatoriness to an unrelated artifact.

## Decision

Add an **orthogonal, optional boolean `mandatory:`** field to `FeatureDef`:

- `mandatory: true` declares the feature's **membership** relationship to its parent — *selected whenever the parent is selected* (parent ⇒ feature); on a top-level feature it means *always selected* (root-mandatory).
- `mandatory: false` or absent means **optional** membership (the default).
- `groupKind:` now governs **child grouping only** (`and` / `or` / `alternative` + `cardinality`). A single `FeatureDef` may therefore be both `mandatory: true` and `groupKind: alternative` — a mandatory XOR group.
- `groupKind: mandatory` is retained as a **backward-compatible shorthand** for mandatory membership on leaves: for any feature that does not set `mandatory:`, `groupKind: mandatory` continues to mean a mandatory member (equivalent to `mandatory: true`).

The change is **additive**: existing models that do not use `mandatory:` behave exactly as before.

## Consequences

- **+** Mandatory groups become **directly expressible** — `mandatory: true` + `groupKind: alternative` is a mandatory XOR group, with no auxiliary node.
- **+** The UAV model is **simplified**: the synthetic `Features::Base` feature (and its `requires:` workaround) is removed, and `Propulsion`, `Payload`, and `DataLink` are marked `mandatory: true` directly.
- **+** Fully **backward compatible** — `groupKind: mandatory` keeps working as a shorthand; models without `mandatory:` are unaffected.
- **−** Two ways now express mandatory-leaf membership (`mandatory: true` and legacy `groupKind: mandatory`); mitigated by treating the latter as a documented compatible shorthand and an advisory (non-blocking) recommendation to prefer `mandatory: true` on nodes that have children.
- This ADR **supersedes nothing**.
