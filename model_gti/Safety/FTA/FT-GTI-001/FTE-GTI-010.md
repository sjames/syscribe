---
id: FTE-GTI-010
type: FaultTreeEvent
eventKind: basic
name: "Perturbation-response check fails silent"
ref: Behavior::Parts::PerturbationCheckPart
failureRate: 5.0e-6
rateSource: "illustrative proposal for architecture comparison (FT-GTI-001)"
rateStatus: unverified
injection: [{kind: silent, part: perturb}]
---

The perturbation-response check (`REQ-GTI-SW-008`) stops: no perturbation is applied and no grid loss is reported from it. Rate as for the other firmware tasks, **unverified**.
