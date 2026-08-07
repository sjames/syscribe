---
type: Configuration
id: CONF-VEHICLE-STD-001
name: "Standard vehicle — fully consolidated"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: CONF-PACK-STD-001
parameterBindings:
  Features::Pack.packCapacityKwh: 75.0
  Features::Cell.manufacturingSiteCode: US
---

The complete, happy-path consolidation. `subConfigurations:` names the battery-pack tier's one
configured variant; that variant is itself internally valid (its own `CONF-CELL-STD-001`
consolidation resolved and SAT-clean) before it can be consolidated here
(`REQ-TRS-HPLE-001`).

Two bindings close everything the subtree still leaves open, at two different depths:

- **`Features::Pack.packCapacityKwh`** — one hop down, the battery-pack tier's own parameter.
- **`Features::Cell.manufacturingSiteCode`** — *two* hops down, reaching straight past the
  battery-pack tier into the battery-cell tier's own `Cell` feature, using its ordinary,
  already-mounted qname — no new addressing syntax, no repo-alias chaining
  (`REQ-TRS-HPLE-002`'s "at any depth" claim, concretely).

With both bound, and `Cell.cycleLifeRating` already closed one tier down by the battery-pack
`Configuration`, and `Cell.nominalVoltageV` never needing anyone's decision at all (its `default:`),
the whole three-tier subtree is fully closed. Validating this `Configuration`'s own model
(`syscribe -m vehicle/model`) reports **zero** `W513` findings for it — see the README.
