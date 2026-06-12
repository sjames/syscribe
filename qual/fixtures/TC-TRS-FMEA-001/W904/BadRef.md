---
type: FMEAEntry
id: FM-TST-001
name: FMEA Entry With Unresolvable Ref
failureMode: Sensor stuck high
effect: System reports incorrect speed
cause: Electrical short
fmeaSeverity: 5
occurrence: 3
detection: 4
rpn: 60
subject: NonExistentElement
---

The `subject` (`ref`) field references `NonExistentElement` which does not resolve to any element — triggers W904.
