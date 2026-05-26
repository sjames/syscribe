---
type: ADR
id: ADR-SYS-PERF-001
name: PerformanceDecompositionADR
title: "Decompose mission performance and regulatory compliance stakeholder goals into measurable system requirements"
status: accepted
tags:
  - performance
  - decomposition
  - regulatory
---

## Context

Mission operators and regulators require the UAV to meet specific operational and regulatory constraints. These stakeholder goals must be broken into verifiable system-level requirements.

## Decision

Two stakeholder-level requirements are introduced:
- **REQ-UAV-PERF-000** — mission performance goal covering endurance, navigation accuracy, and data link range.
- **REQ-UAV-REG-000** — regulatory compliance goal covering maximum take-off mass.

Each is decomposed into one leaf requirement assigned to a specific architecture element:
- REQ-UAV-ENDUR-001 → `UAV::Power::BatteryPack` (hardware, endurance)
- REQ-UAV-NAV-001 → `UAV::Avionics::GPSReceiver` (hardware, GPS accuracy)
- REQ-UAV-COMM-001 → `UAV::Avionics::FlightController` (software, telemetry link)
- REQ-UAV-MASS-001 → `UAV::UAVSystem` (system, mass compliance)

## Consequences

All four leaf requirements are verifiable by test or analysis. The parent requirements are satisfied when their children are verified.
