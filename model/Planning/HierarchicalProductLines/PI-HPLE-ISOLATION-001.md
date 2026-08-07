---
type: PlanningItem
id: PI-HPLE-ISOLATION-001
name: "Confirm no mechanism lets a lower tier reference or become aware of a higher tier"
status: done
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-005]
evidence:
  - path: "repo:crates/syscribe-model/tests/hple_isolation.rs"
tags:
  - variability
  - multi-repo
---

Architectural/structural rather than independently unit-testable in isolation, per
`REQ-TRS-HPLE-005`'s own scope — its observable consequence is that `PI-HPLE-SUBCONFIG-001`,
`PI-HPLE-PARAMBIND-001`, and `PI-HPLE-BINDGUARD-001`'s own tests never require, accept, or resolve
an upward-pointing reference from a descendant. Evidence here should point at a specific
regression test (in one of those three items' own test suites, or a dedicated one) that positively
confirms `bindTo:` cannot be repurposed to cross a `subConfigurations:` boundary, rather than
inventing new mechanism-building work of its own.

Confirmed by inspection first: `bindTo:`'s only two checks (`E202` propagation-range, `E229`
binding-time ordering) live entirely in `feature_model::check_feature_model`, which — like every
feature-model function in this codebase — takes exactly one `elements` slice per call. Every call
site (`validate_with_config`'s peer-validity gate, `feature-check`, `build_config.rs`) passes either
a single model's own `elements` or one specific peer's own `walk_model` result — never a
concatenation of two repos' elements. A descendant's `bindTo:` therefore cannot even *see* a
consolidating tier's parameters to name in the first place, by construction, independent of whether
the dotted string it names happens to collide with something real one tier up.

`hple_isolation.rs` demonstrates this concretely rather than resting on inspection alone: a lower
("battery") tier's `Cell.voltage` parameter declares `bindTo: "Features::TopSecret.forbidden"` — a
path naming nothing in its own model but coincidentally matching a real parameter in a separate,
higher ("vehicle") tier that consolidates it via `subConfigurations:`. Three tests — a positive
control confirming the mechanism fires correctly on a genuinely local match; confirming the higher
tier's own validation raises zero `E202` regardless of what value it binds to the colliding key
(the lower tier's `bindTo`/`range` metadata is structurally absent from its `elements`); and the
symmetric case, confirming the lower tier's own independent validation is unaffected by the sibling
higher-tier model's bindings existing on disk at all.
