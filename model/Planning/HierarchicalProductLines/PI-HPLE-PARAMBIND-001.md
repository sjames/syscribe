---
type: PlanningItem
id: PI-HPLE-PARAMBIND-001
name: "parameterBindings: reaches transitively through a consolidated subtree via ordinary qname resolution"
status: done
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-002]
evidence:
  - path: "repo:crates/syscribe-model/tests/hple_parambind.rs"
tags:
  - variability
  - multi-repo
---

Extend `parameterBindings:` resolution so a dotted key can target a parameter reachable through
`subConfigurations:` at any depth, using the parameter's ordinary already-mounted qname — no new
addressing syntax.

`parameter_binding_findings` now falls back, on a local-lookup miss, to
`collect_reachable_feature_params` — a recursive walk mirroring `sub_configuration_findings`'s own
local-then-peer, dependency-ordered traversal, but built once, lazily, per `Configuration` rather
than eagerly for the whole model. Each tier's own `[repos]` table governs its own next hop (never
inherited from the caller), so a chain of any depth resolves without a new addressing scheme.

The intrinsic per-parameter checks (`E204` fixed, `E205` range, `E206` enum, `W027` runtime
binding-time) apply identically whether a binding's target is local or transitively resolved.
`E203` (feature not selected) and the required-and-unbound `W017` sweep stay scoped to this
`Configuration`'s own local feature-selection map — a transitively-reached parameter's selection
state belongs to whichever tier actually selects it, and which cross-tier bindings are *permitted*
(as opposed to merely *resolvable*) is `PI-HPLE-BINDGUARD-001`'s job (`REQ-TRS-HPLE-003`), not this
one's.

**Correction found while writing the local-subConfigurations regression test:** a first draft
exempted every locally-resolved dotted key from `E203` once a `Configuration` had *any*
`subConfigurations:` at all, reasoning by analogy with the peer case. That's wrong: within one
repo there is exactly one feature model/SAT instance (`check_feature_model[_deep]` is not scoped
per `Configuration`), so a `Configuration` genuinely either selects a locally-reachable feature or
it doesn't — `E203` stays fully meaningful for a local target. The distinction that actually
matters is the repo boundary itself: a peer `Configuration`'s selected features live in a
genuinely separate feature model the consolidating tier cannot see into, which is exactly why
`E203` must not apply there. Fixed by scoping the `E203` exemption to parameters actually resolved
through the transitive (subConfigurations-reachability) walk, not to every locally-found parameter
merely because *some* `subConfigurations:` entry exists on the same `Configuration`.

8 tests: one-tier and two-tier (nested `[repos]`) cross-repo resolution; a negative control (a
loaded-but-not-consolidated peer repo's `FeatureDef` must stay unresolved); `E204`/`E205`/`E206`
firing correctly on a transitively-resolved parameter; `E203` never firing on one; and the local
regression guard above.
