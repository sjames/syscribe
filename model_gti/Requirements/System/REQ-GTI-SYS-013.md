---
type: Requirement
id: REQ-GTI-SYS-013
name: "Remote active-power curtailment"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
  - REQ-GTI-STK-005
breakdownAdr: ADR-GTI-REQ-001
appliesWhen: Features::GridSupport::RemoteCurtailment
---

On an authenticated command the inverter **shall** limit active power to the commanded percentage of rated power within 10 s and hold it until changed.
