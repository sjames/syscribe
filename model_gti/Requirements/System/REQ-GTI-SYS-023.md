---
type: Requirement
id: REQ-GTI-SYS-023
name: "Global maximum search under partial shading"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-002
breakdownAdr: ADR-GTI-REQ-001
appliesWhen: Features::Mppt::GlobalScan
---

A tracker **shall** periodically sweep the whole window to find the global maximum of a multi-peak P-V curve and move to it if it exceeds the current point by more than the hysteresis; the period is the `scanPeriodS` parameter.
