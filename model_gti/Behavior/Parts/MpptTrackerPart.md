---
type: PartDef
name: MpptTrackerPart
exhibitsStates: [Behavior::Machines::MpptTracker]
models: [System::Software::MpptController]
fidelity: "perturb-and-observe tracking state only"
abstracts: [ctl, stringSamples, powerRef, stringData]
---

One MPPT tracker instance (one per boost stage, REQ-GTI-SW-006).
