---
id: FTG-GTI-010
type: FaultTreeGate
gateType: OR
name: "Monitor missed the loss"
inputs: [FTE-GTI-004, FTE-GTI-007, FTE-GTI-009]
---

The voltage/frequency monitor does not report the loss: it has failed silent, or the software sensing chain (alone or with the hardware one) reads nominal.
