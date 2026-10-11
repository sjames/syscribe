---
type: PartDef
name: Arc-Fault Analyser
domain: software
silLevel: 2
appliesWhen: Features::Protection::Afci
satisfies:
  - REQ-GTI-SW-004
features:
  - {name: arcSamples, type: Port, direction: in}
  - {name: trip, type: Port, direction: out}
---

Spectral signature classification of the arc-fault front-end signals.
