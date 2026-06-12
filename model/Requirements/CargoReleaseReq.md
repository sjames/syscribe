---
type: Requirement
id: REQ-UAV-CARGO-001
name: "Delivery payload shall release cargo within 500 ms of command"
status: approved
reqDomain: hardware
verificationMethod: test
appliesWhen: Features::Payload::Delivery
derivedFrom:
  - REQ-UAV-VAR-000
breakdownAdr: Decisions::ProductLineADR
tags:
  - delivery
  - payload
---

When configured with the cargo delivery payload, the UAV shall actuate the cargo
release mechanism within 500 ms of receiving a validated release command, and
shall confirm release over telemetry.

## Rationale

Accurate drop timing bounds the landing dispersion of delivered cargo. This
requirement is only present in products that select the `Delivery` payload.
