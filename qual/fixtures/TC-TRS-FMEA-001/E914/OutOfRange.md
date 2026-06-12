---
type: FMEAEntry
id: FM-TST-001
name: FMEA Entry With Out Of Range Values
failureMode: Sensor stuck high
effect: System reports incorrect speed
cause: Electrical short
fmeaSeverity: 11
occurrence: 0
detection: 10
rpn: 1100
---

The `fmeaSeverity` value 11 and `occurrence` value 0 are both outside range 1–10 — triggers E914.
