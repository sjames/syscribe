---
type: PlanningItem
id: PI-HPLE-BINDGUARD-001
name: "Reject illegal/redundant cross-tier parameterBindings: entries"
status: done
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-003]
evidence:
  - path: "repo:crates/syscribe-model/tests/hple_bindguard.rs"
tags:
  - variability
  - multi-repo
  - validation
---

Extend the existing single-model `E203`/`E204` reasoning ("must not bind a parameter of an
unselected feature" / "must not bind a fixed parameter") across the `subConfigurations:` boundary,
plus rejecting a double-bind of a parameter already closed by a nearer tier.

Two new codes, continuing the `E516`–`E518` HPLE cluster rather than overloading `E203`/`E204`
themselves (`E204`'s "fixed parameter" reasoning already applies uniformly to transitive bindings
unchanged, from `PI-HPLE-PARAMBIND-001` — nothing new needed there):

- **`E519`** — a transitively-resolved binding targets a `FeatureDef` the owning peer
  `Configuration` doesn't itself select. `collect_reachable_feature_params` (extended into
  `TransitiveParamStatus`) now also computes each reachable parameter's owning peer's own
  `canon_selection`, tagging `selected_by_owner` at the exact hop where a peer's `build_feature_params`
  match is merged — the only hop that structurally *can* own that `FeatureDef`.
- **`E523`** — a transitively-resolved binding double-binds something a nearer tier already
  supplies. Every hop on the walk, after its own recursive call returns, checks its *own*
  `parameterBindings:` against everything the deeper recursion found reachable
  (`mark_already_bound_by`) — since this runs on the way back up (post-order), a tier nearer the
  querying `Configuration` overwrites a farther one, so the reported tier is always the nearest
  one that actually closed it, verified adversarially against a 3-tier chain where both an
  intermediate and the owner bind the same parameter.

Both checks are scoped to transitively-resolved bindings only (`is_transitive`, from
`PI-HPLE-PARAMBIND-001`) — a purely local `subConfigurations:` chain shares one feature model per
repo, where plain `E203` already applies correctly and unchanged.

5 tests in `hple_bindguard.rs`: `E519` firing/not-firing on an unselected/selected owner; `E523`
firing on the owner's own double-bind, on a local intermediate tier's, and (the adversarial case)
correctly naming the *nearer* of two tiers that both bind the same parameter rather than the
deeper owner.
