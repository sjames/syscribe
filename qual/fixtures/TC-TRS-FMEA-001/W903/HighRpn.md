---
type: FMEAEntry
id: FM-TST-001
title: FMEA Entry With High RPN And No Action
failureMode: Motor controller overtemperature
effect: Motor shutdown during operation
cause: Cooling fan failure
fmeaSeverity: 10
occurrence: 10
detection: 10
rpn: 1000
---

RPN is 1000 (> 100) and there is no `recommendedAction` field — triggers W903.
