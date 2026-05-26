---
type: ADR
id: ADR-SYS-SAFE-001
name: SafetyDecompositionADR
title: "Decompose UAV safety requirement into fault detection and safe landing sub-requirements"
status: accepted
tags:
  - safety
  - decomposition
---

## Context

REQ-UAV-SAFE-000 mandates that the UAV shall not cause injury during any flight phase. To make this requirement verifiable, it must be decomposed into concrete, testable sub-requirements assigned to specific architecture elements.

## Decision

Decompose REQ-UAV-SAFE-000 into two leaf requirements:
1. **REQ-UAV-FC-001** — The flight controller shall detect any single sensor failure within 50 ms. Assigned to `UAV::Avionics::FlightController` (software domain, ASIL-C).
2. **REQ-UAV-SAFE-001** — The UAV shall execute autonomous safe landing on battery-critical or link-loss event. Assigned to `UAV::Avionics::FlightController` (software domain, ASIL-B).

## Consequences

Both requirements are verifiable via HIL test on the fault-injection bench. The parent REQ-UAV-SAFE-000 is deemed satisfied when both children are verified.
