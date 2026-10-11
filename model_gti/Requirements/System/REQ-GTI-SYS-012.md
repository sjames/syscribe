---
type: Requirement
id: REQ-GTI-SYS-012
name: "Frequency-watt support"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
breakdownAdr: ADR-GTI-REQ-001
tags: [reg-unverified]
appliesWhen: Features::GridSupport::FreqWatt
---

With frequency-watt enabled the inverter **shall** reduce active power as grid frequency rises above the dead band of the profile.
