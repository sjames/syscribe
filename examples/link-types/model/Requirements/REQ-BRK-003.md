---
type: Requirement
id: REQ-BRK-003
name: "Hydraulic pressure released within 10 ms of a release command"
status: approved
reqDomain: hardware
reqClass: derived
derivedFrom: [REQ-BRK-001]
breakdownAdr: Decisions::ADR-BRK-001
tags:
  - brakes
  - safety
---

The hydraulic modulator shall reduce caliper pressure by at least 80 percent
within 10 ms of receiving a pressure-release command.

## Notes

Satisfied by `Architecture::HydraulicModulator` (hardware, `satisfies:`).
`Architecture::BrakeController` (software) contributes through the
project-defined `partiallySatisfies` link, which relaxes `E313` and earns no
coverage credit. `Architecture::WatchdogMonitor` and
`Architecture::PressureReliefValve` hold `mitigates` links to it.
