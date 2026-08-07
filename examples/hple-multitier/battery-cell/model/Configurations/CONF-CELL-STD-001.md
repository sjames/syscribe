---
type: Configuration
id: CONF-CELL-STD-001
name: "Standard cell chemistry"
status: approved
featureModel: Features
features:
  Features: true
  Features::Cell: true
---

The one cell-chemistry variant this leaf tier offers. Selects `Cell` and supplies no
`parameterBindings:` at all — `nominalVoltageV` needs none (it has a `default:`), and
`cycleLifeRating`/`manufacturingSiteCode` are exactly the parameters this tier deliberately leaves
open for whoever consolidates it. This `Configuration` is independently valid on its own (SAT-clean,
zero errors) — `REQ-TRS-HPLE-001`'s "consolidation of *configured* lower-tier models" requires
exactly that, and validating this model on its own (`syscribe -m battery-cell/model`) confirms it.
