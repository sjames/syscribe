---
type: Requirement
id: REQ-GTI-SYS-011
name: "Volt-var support"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
breakdownAdr: ADR-GTI-REQ-001
tags: [reg-unverified]
appliesWhen: Features::GridSupport::VoltVar
---

With volt-var enabled the inverter **shall** adjust reactive power as a function of grid voltage along the curve set in the profile.
