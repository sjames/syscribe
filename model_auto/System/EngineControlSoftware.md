---
type: PartDef
name: Engine Control Software
domain: software
isDeploymentPackage: true
features:
  - name: softwareVersion
    type: ScalarValues::String
  - name: buildDate
    type: ScalarValues::String
---

The Engine Control Software is the complete software image deployed to the
Engine ECU hardware. It encompasses all functional components: throttle control,
fuel control, safety monitor, and CAN security module.

## Software architecture

Structured as an AUTOSAR-compliant layered architecture:
- **Application layer** — functional components (ThrottleControl, FuelControl, SafetyMonitor)
- **RTE** — AUTOSAR run-time environment for inter-component communication
- **BSW** — basic software (MCAL, OS, COM stack, diagnostic services)

## Deployment

Deployed to the `System::EngineECU` hardware. Allocation documented in
`Allocations::SwToECU`.
