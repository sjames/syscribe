---
type: Diagram
name: GtiPowerView
diagramKind: IBD
subject: System::GridTieInverter
include: {kinds: [power]}
style: focus
layout: {flow: left-to-right, layers: [power, sensing]}
---

The power path, from the PV input through the power stage to the AC output. Derived from `System::GridTieInverter`: the connections that carry `flowKind: power` and the parts they join are drawn, everything else faded. `layers:` keeps the power parts in the first band and the sensing part below them.
