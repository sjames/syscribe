---
type: Configuration
id: CONF-PACK-STD-001
name: "Standard battery pack"
status: approved
featureModel: Features
features:
  Features: true
  Features::Pack: true
subConfigurations: CONF-CELL-STD-001
parameterBindings:
  Features::Cell.cycleLifeRating: 1000
---

Consolidates the `battery-cell` line's one configured variant (`REQ-TRS-HPLE-001`: `CONF-CELL-STD-001`
is itself independently valid, resolved here purely by its stable id — no repo-alias-chained
addressing). Its `parameterBindings:` closes exactly one of the two parameters `Cell` leaves open —
`cycleLifeRating` — using the parameter's ordinary, already-mounted qname
(`REQ-TRS-HPLE-002`): a pack-design decision this tier is genuinely positioned to make.

`manufacturingSiteCode` and this tier's own `packCapacityKwh` are deliberately left unbound —
sourcing site and total pack energy are both vehicle-program decisions this tier cannot and should
not make on its own behalf. Left open at an intermediate tier, this is not a defect: running
`syscribe -m battery-pack/model` on its own reports these only as the opt-in `W513`
(`REQ-TRS-HPLE-004`), never a hard error — see the README for the exact command and output.
