---
type: Requirement
id: REQ-ENG-PERF-001
name: Throttle position shall track pedal demand within 50 ms
status: approved
reqDomain: software
derivedFrom:
  - REQ-ENG-PERF-000
breakdownAdr: ADR-ENG-PERF-001
verificationMethod: test
---

The electronic throttle control **shall** achieve the commanded throttle position
within 50 ms of a step change in accelerator pedal demand, measured from pedal
signal crossing 10 % to throttle plate reaching 90 % of the commanded position,
across the full operating temperature range (−40 °C to +125 °C).
