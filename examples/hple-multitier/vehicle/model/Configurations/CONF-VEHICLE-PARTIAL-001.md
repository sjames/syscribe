---
type: Configuration
id: CONF-VEHICLE-PARTIAL-001
name: "Vehicle program in progress — sourcing site not yet decided"
status: draft
featureModel: Features
features:
  Features: true
subConfigurations: CONF-PACK-STD-001
parameterBindings:
  Features::Pack.packCapacityKwh: 100.0
---

A second, deliberate variant demonstrating `REQ-TRS-HPLE-004`: this vehicle program has settled on
pack capacity but has not yet decided a manufacturing sourcing site, so
`Features::Cell.manufacturingSiteCode` stays unbound here — on purpose, mid-program, not a defect.

Validating this model reports exactly **one** `W513`, naming `Features::Cell.manufacturingSiteCode`
still open in the consolidated subtree — a warning, silent to the exit code unless this repo's own
CI explicitly opts in with `--deny W513` (which it would not, here, since this repo is a worked
example, not the actual point of final assembly for anyone's product). See the README for the exact
command and output.
