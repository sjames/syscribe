---
type: PartDef
name: SensorPart
exhibitsStates: [Behavior::Machines::Sensor]
models: [System::Hardware::Sensing]
fidelity: "one measurement chain; the two chains are two instances (see ProtectionChain)"
abstracts: [dcSig, tempSig]
portMap:
  out: adcOut   # Sensing has two SamplePort outputs, so the shared type alone is ambiguous
features:
  - {name: inp, type: Port, direction: in, typedBy: Interfaces::SamplePort, isConjugated: true}
  - {name: out, type: Port, direction: out, typedBy: Interfaces::SamplePort}
---

Sensing chain.
