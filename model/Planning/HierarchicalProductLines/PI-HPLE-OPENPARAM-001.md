---
type: PlanningItem
id: PI-HPLE-OPENPARAM-001
name: "Opt-in, --deny-gateable warning for an unresolved required parameter anywhere in a consolidated subtree"
status: done
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-004]
evidence:
  - path: "repo:crates/syscribe-model/tests/hple_openparam.rs"
tags:
  - variability
  - multi-repo
  - validation
---

Compute the transitive closure of unbound `isRequired: true` parameters across an entire
consolidated subtree; report as a warning (never a hard error at an intermediate tier's own
isolated validation run), following the existing `W510`/`W511`/`W512`/`W023`/`W090` opt-in,
`--deny`-gateable posture.

New code `W513`, implemented as `open_parameter_findings` — a thin layer over
`collect_reachable_feature_params`'s existing walk (already computes `selected_by_owner` and
`already_bound_by` per reachable parameter, built for `PI-HPLE-BINDGUARD-001`). "Still open" is
exactly: selected by its owner, required, no default, not `bindingTime: runtime`, `already_bound_by`
still `None`, and not in this `Configuration`'s own `parameterBindings:` either (the one check the
reused walk doesn't already do, since it never inspects the *querying* `Configuration`'s own
bindings — only tiers strictly nearer than it). A purely local `subConfigurations:` chain
contributes nothing to the walk at all (per that function's own doc comment), so `W513` stays
silent there too, matching the reasoning already established for `E519`/`E523`: one shared feature
model per repo has plain `W017` already covering it, unconditionally.

7 tests in `hple_openparam.rs`: fires on a genuinely open peer parameter; suppressed when closed by
the querying `Configuration` itself, by a nearer local intermediate tier, or by an owning tier two
hops down; never fires for an unselected feature or for fixed/runtime parameters; and confirmed
`Warning`-severity (never escalated to a hard error on its own).
