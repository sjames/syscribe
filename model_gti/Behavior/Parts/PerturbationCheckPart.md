---
type: PartDef
name: PerturbationCheckPart
exhibitsStates: [Behavior::Machines::PerturbationCheck]
models: [System::Software::ProtectionManager]
fidelity: "perturbation-response check only; the injection is a current step, not a reactive-current or phase step"
abstracts: [arcTrip, profile, relayCmd]
portMap:
  req: null   # the measurement request is how the test set-up samples on demand; the firmware reads its ADC
features:
  - {name: samples, type: Port, direction: in, typedBy: Interfaces::SamplePort, isConjugated: true}
  - {name: req, type: Port, direction: out, typedBy: Interfaces::ProbePort}
  - {name: trip, type: Port, direction: out, typedBy: Interfaces::GridStatusPort}
---

Anti-islanding perturbation-response check.
