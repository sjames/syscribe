---
type: PlanningItem
id: PI-HPLE-EXAMPLE-001
name: "Build a comprehensive worked example (multi-tier consolidation)"
status: done
itemType: task
parent: PI-HPLE-001
evidence:
  - path: "repo:examples/hple-multitier/README.md"
tags:
  - variability
  - multi-repo
---

A realistic multi-repo example (mirroring `examples/sysmlv2-submodel/` and
`examples/planning-item/`'s structure): at least 3 tiers, `subConfigurations:` at the top two,
`parameterBindings:` closing some parameters at the immediate consolidating tier and deliberately
deferring others further up (demonstrating transitive pass-through), and at least one
intentionally-`default:`-supplied parameter that never needs external injection at all.

`examples/hple-multitier/{battery-cell,battery-pack,vehicle}/model/` — three genuinely separate
model roots (unlike the single-root `sysmlv2-submodel`/`planning-item` examples, since demonstrating
this feature requires real `[repos]` boundaries), connected only by `[repos]` + `subConfigurations:`:

- `vehicle` consolidates `battery-pack`, which itself consolidates `battery-cell` —
  `subConfigurations:` at the top two tiers, none at the leaf.
- `battery-pack` closes `Cell.cycleLifeRating` one hop down, right where it's declared.
- `vehicle` closes `Cell.manufacturingSiteCode` **two** hops down, straight past `battery-pack`,
  demonstrating `REQ-TRS-HPLE-002`'s "at any depth" claim concretely, plus its own
  `Pack.packCapacityKwh` one hop down.
- `Cell.nominalVoltageV` carries a `default:` and is never touched by any `parameterBindings:` —
  the intentionally-self-sufficient parameter.
- Two `vehicle` Configurations: `CONF-VEHICLE-STD-001` (fully closed, 0 `W513`) and
  `CONF-VEHICLE-PARTIAL-001` (deliberately leaves `manufacturingSiteCode` open, exactly 1 `W513`) —
  demonstrating `REQ-TRS-HPLE-004`'s opt-in completeness warning concretely, including
  `--deny W513` actually gating (exit 2).

Each tier validates cleanly on its own (0 errors); a real authoring bug was caught and fixed while
building this — `battery-pack`'s own `Configuration` initially selected only its `Features` root,
never its own mandatory `Pack` child, which correctly surfaced as `E518`/`E519`/`E225` once `vehicle`
tried to consolidate it (a locally-invalid Configuration cannot be consolidated,
`REQ-TRS-HPLE-001`) — fixed by adding the missing `Features::Pack: true` selection.
