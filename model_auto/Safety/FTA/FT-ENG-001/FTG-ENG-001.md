---
type: FaultTreeGate
id: FTG-ENG-001
name: OR gate — throttle control failure OR sensor input error
gateType: OR
inputs:
  - FTG-ENG-002
  - FTE-ENG-003
probability: 2.4e-7
---

The top-level OR gate: unintended acceleration occurs if either the sensor
input chain fails (FTG-ENG-002) OR the throttle actuator fails stuck open
(FTE-ENG-003).
