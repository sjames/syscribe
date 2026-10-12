---
type: PortDef
name: SamplePort
features:
  - {name: sample, typedBy: Interfaces::GridSample, direction: out}
  - {name: probe, typedBy: Interfaces::Probe, direction: out}
---

Grid measurements, flowing out of the sensing side. The consuming port is conjugated.
