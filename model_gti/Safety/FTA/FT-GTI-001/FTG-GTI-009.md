---
id: FTG-GTI-009
type: FaultTreeGate
gateType: AND
name: "Both software detections missed the loss"
inputs: [FTG-GTI-010, FTE-GTI-010]
---

The software misses a grid loss only when the voltage/frequency monitor misses it **and** the perturbation-response check does too.
