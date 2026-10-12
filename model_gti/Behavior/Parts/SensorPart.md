---
type: PartDef
name: SensorPart
exhibitsStates: [Behavior::Machines::Sensor]
models: [System::Hardware::Sensing]
fidelity: "one measurement chain; the two chains are two instances (see ProtectionChain)"
abstracts: [dcSig, tempSig]
portMap:
  inp: acSig
  out: adcOut
features:
  - {name: inp, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---

Sensing chain.
