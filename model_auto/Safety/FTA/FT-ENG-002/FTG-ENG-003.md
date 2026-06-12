---
type: FaultTreeGate
id: FTG-ENG-003
name: OR gate — CPS signal lost due to harness failure OR supply dropout
gateType: OR
inputs:
  - FTG-ENG-004
  - FTE-ENG-004
  - FTE-ENG-005
probability: 1.2e-4
---

The top-level OR gate for the engine stall fault tree: the crankshaft position
signal is entirely lost if any of the following occur:

- The CPS wire harness suffers an open circuit (FTE-ENG-004) — the dominant
  failure path at 1.2e-6/h
- The ECU supply voltage drops below 7 V (FTE-ENG-005) — causing a full ECU
  reset and loss of CPS processing
- Both CPS signal degradation and an EMC burst occur simultaneously
  (FTG-ENG-004 AND gate) — extremely rare due to shielding

The overall OR gate probability is dominated by the harness open-circuit path
(FTE-ENG-004), which accounts for approximately 93 % of the total.
