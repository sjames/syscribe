---
type: FeatureDef
id: FEAT-V7-WDT
name: Wdt
groupKind: optional
parameters:
  - name: timeoutMs
    type: ScalarValues::Integer
    range: "10..1000"
    isRequired: true
---
Optional watchdog with a required timeout.
