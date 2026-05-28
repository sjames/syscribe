---
type: PartDef
name: Engine Stall Monitor
domain: software
asilLevel: B
satisfies:
  - REQ-ENG-SAFE-003
features:
  - name: stallRpmThreshold
    type: ScalarValues::Integer
    unit: rpm
  - name: warningTimeMs
    type: ScalarValues::Integer
    unit: ms
---

Software component monitoring crankshaft position signal integrity and engine
speed to detect conditions leading to critical engine stall (ASIL B).

Monitors CPS signal validity at every ignition event. On CPS signal loss,
the component initiates a controlled deceleration sequence before stall occurs,
alerting the driver and setting a DTC.
