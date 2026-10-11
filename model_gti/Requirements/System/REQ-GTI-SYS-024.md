---
type: Requirement
id: REQ-GTI-SYS-024
name: "String monitoring"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-002
breakdownAdr: ADR-GTI-REQ-001
appliesWhen: Features::Mppt::StringMonitoring
---

The inverter **shall** measure each string current and raise a warning when a string produces less than 70 % of the median of its tracker's strings for longer than 15 min in irradiance above a threshold.
