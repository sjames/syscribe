---
type: FaultTreeGate
id: FTG-ENG-002
title: AND gate — simultaneous CPS and TPS failure defeats redundancy
gateType: AND
inputs:
  - FTE-ENG-001
  - FTE-ENG-002
probability: 1.5e-10
---

Both the crankshaft position sensor AND the throttle position sensor must fail
simultaneously for sensor-based throttle demand to be undetectably incorrect.
The safety monitor's cross-checks detect any single sensor failure; only the
AND combination defeats this protection.
