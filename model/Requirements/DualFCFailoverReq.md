---
type: Requirement
id: REQ-UAV-REDUN-001
title: "Backup flight controller shall assume control within 100 ms of primary loss"
status: approved
reqDomain: software
verificationMethod: test
asilLevel: C
appliesWhen: Features::DualFlightController
derivedFrom:
  - REQ-UAV-VAR-000
breakdownAdr: Decisions::ProductLineADR
tags:
  - redundancy
  - safety
  - flight-controller
---

When configured with dual flight controllers, the backup flight controller shall
detect loss of the primary controller's heartbeat and assume control authority
within 100 ms, with no loss of attitude stabilisation.

## Rationale

Redundant builds must tolerate a single flight-controller failure without
departure from controlled flight. This requirement is only present in products
that select the `DualFlightController` feature.
