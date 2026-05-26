# UAV Model

This directory contains the Syscribe model for an autonomous UAV flight system. Each `.md` file is a model element; the directory tree encodes the namespace hierarchy. YAML frontmatter holds structured metadata; the Markdown body is the element's documentation.

---

## Top-level namespace layout

```
model/
├── UAV/               — structural architecture (PartDefs, sub-systems)
│   ├── Avionics/      — FlightController, GPSReceiver, IMU, AvionicsBay
│   ├── Power/         — BatteryPack, PowerDistributionUnit, PowerSystem
│   ├── Propulsion/    — Motor, RotorAssembly, PropulsionSystem, rotor configs
│   └── Payload/       — Camera, PayloadBay
├── GroundStation/     — GroundControlStation, OperatorConsole
├── Behavior/          — MissionExecution, TakeoffAction, WaypointNavAction,
│                        LandingAction, FlightStates
├── Interfaces/        — port defs, connection defs, interface blocks
├── Flows/             — PowerFlowDef, TelemetryFlowDef
├── Items/             — item types flowing through the system
├── Enumerations/      — FlightMode, ArmStatus
├── Requirements/      — requirement hierarchy (REQ-*)
├── Verification/      — test cases (TC-*) and review records
├── Calculations/      — performance trade calculations
├── Constraints/       — parametric constraints
├── Allocations/       — function and requirement allocations
├── Decisions/         — architecture decision records (ADR-*)
├── UseCases/          — actor-level use cases
├── Views/             — model views
├── Viewpoints/        — stakeholder viewpoints
├── Metadata/          — cross-cutting annotations
└── Diagrams/          — all diagrams (BDD, IBD, Sequence, State Machine)
```

---

## Architecture overview

`UAV::UAVSystem` is the abstract top-level `PartDef`. Concrete configurations specialise it. Five sub-systems compose the vehicle:

| Sub-system | Qualified name | Key elements |
|---|---|---|
| Avionics | `UAV::Avionics` | `FlightController`, `GPSReceiver`, `IMU`, `AvionicsBay` |
| Power | `UAV::Power` | `BatteryPack`, `PowerDistributionUnit`, `PowerSystem` |
| Propulsion | `UAV::Propulsion` | `Motor`, `RotorAssembly`, `PropulsionSystem`, `QuadRotorConfig`, `HexRotorConfig` |
| Payload | `UAV::Payload` | `PayloadBay`, `Camera` |
| Airframe | `UAV` | `Airframe` |

The ground segment is defined under `GroundStation::GroundControlStation` and interacts with the UAV via the `TelemetryConnectionDef` interface.

---

## Diagrams

| Diagram | Kind | Subject | Notes |
|---|---|---|---|
| [UAVSystemBDD](Diagrams/UAVSystemBDD.md) | BDD | `UAV::UAVSystem` | Top-level block decomposition |
| [PropulsionSystemBDD](Diagrams/PropulsionSystemBDD.md) | BDD | `UAV::Propulsion` | Motor / RotorAssembly composition |
| [AvionicsBayIBD](Diagrams/AvionicsBayIBD.md) | IBD | `UAV::Avionics::AvionicsBay` | Internal connections within avionics bay |
| [PowerSystemIBD](Diagrams/PowerSystemIBD.md) | IBD | `UAV::Power::PowerSystem` | Battery → PDU power flow |
| [MissionExecutionSeq](Diagrams/MissionExecutionSeq.md) | Sequence | `Behavior::MissionExecution` | Full mission message flow (GCS ↔ FC ↔ Prop ↔ GPS) |
| [FlightStatesMachineD](Diagrams/FlightStatesMachineD.md) | StateMachine | `Behavior::FlightStates` | Flight phase state machine |
| [SafetyRequirementsD](Diagrams/SafetyRequirementsD.md) | Custom | Safety requirements | Requirement / allocation / verification trace |
| [RequirementTraceMermaid](Diagrams/RequirementTraceMermaid.md) | Mermaid | `Requirements` | Full requirement traceability graph |

---

## Requirements

### Parent requirements

| ID | Title | Domain |
|---|---|---|
| REQ-UAV-SAFE-000 | UAV shall not cause injury to persons or property during any flight phase | — |
| REQ-UAV-PERF-000 | UAV shall meet mission performance objectives for endurance, navigation, and data link | — |
| REQ-UAV-REG-000 | UAV shall comply with all applicable regulatory requirements (sub-5 kg open category) | — |

### Leaf requirements

| ID | Title | Domain | Status |
|---|---|---|---|
| REQ-UAV-MASS-001 | Maximum take-off mass ≤ 5 kg | system | approved |
| REQ-UAV-ENDUR-001 | Minimum 25-minute flight endurance under nominal conditions | hardware | approved |
| REQ-UAV-NAV-001 | GPS position accuracy ≤ 1.5 m CEP under nominal GNSS | hardware | approved |
| REQ-UAV-COMM-001 | Telemetry data link range ≥ 5 km line of sight | software | approved |
| REQ-UAV-FC-001 | Flight controller shall detect single sensor failure within 50 ms | software | approved |
| REQ-UAV-SAFE-001 | Autonomous safe landing on battery-critical or link-loss event | software | approved |

---

## Verification

Nine test cases in `Verification/` cover the leaf requirements:

| File | Level | Verifies |
|---|---|---|
| [MassVerificationCase](Verification/MassVerificationCase.md) | L1 | REQ-UAV-MASS-001 |
| [EnduranceTestCase](Verification/EnduranceTestCase.md) | L2 | REQ-UAV-ENDUR-001 |
| [NavigationAnalysisCase](Verification/NavigationAnalysisCase.md) | L3 | REQ-UAV-NAV-001 |
| [DataLinkTestCase](Verification/DataLinkTestCase.md) | L2 | REQ-UAV-COMM-001 |
| [FCFaultInjectionTest](Verification/FCFaultInjectionTest.md) | L4 | REQ-UAV-FC-001 |
| [SafeLandingTest](Verification/SafeLandingTest.md) | L4 | REQ-UAV-SAFE-001 |
| [PerformanceCaseReview](Verification/PerformanceCaseReview.md) | L1 | REQ-UAV-PERF-000 |
| [SafetyCaseReview](Verification/SafetyCaseReview.md) | L1 | REQ-UAV-SAFE-000 |
| [RegulatoryComplianceReview](Verification/RegulatoryComplianceReview.md) | L1 | REQ-UAV-REG-000 |

---

## Architecture decisions

| ID | File | Status | Subject |
|---|---|---|---|
| ADR-SYS-001 | [SafetyDecompositionADR](Decisions/SafetyDecompositionADR.md) | accepted | Safety requirement decomposition strategy |
| ADR-SYS-002 | [PerformanceDecompositionADR](Decisions/PerformanceDecompositionADR.md) | accepted | Performance requirement decomposition strategy |

---

## Behavior

Mission execution is modelled under the `Behavior` namespace:

- **`MissionExecution`** — top-level action def; orchestrates the three flight sub-actions
- **`TakeoffAction`** — throttle ramp and altitude monitoring loop
- **`WaypointNavAction`** — GPS-guided waypoint traversal loop
- **`LandingAction`** — descent, altitude monitoring, and motor disarm
- **`FlightStates`** — state machine: `Idle → Arming → Takeoff → Cruise → Landing → Disarmed`

---

## Interfaces and flows

| Element | Kind | Carries |
|---|---|---|
| `Interfaces::TelemetryPortDef` | PortDef | `Items::TelemetryPacket` |
| `Interfaces::ControlPortDef` | PortDef | `Items::ControlCommand` |
| `Interfaces::PowerPortDef` | PortDef | `Items::BatteryPower` |
| `Interfaces::TelemetryConnectionDef` | ConnectionDef | Telemetry flow GCS ↔ UAV |
| `Interfaces::PowerConnectionDef` | ConnectionDef | DC power distribution |

---

## Navigating the model

- **Start with structure**: [UAVSystemBDD](Diagrams/UAVSystemBDD.md) → [AvionicsBayIBD](Diagrams/AvionicsBayIBD.md) → [PowerSystemIBD](Diagrams/PowerSystemIBD.md)
- **Understand behavior**: [FlightStatesMachineD](Diagrams/FlightStatesMachineD.md) → [MissionExecutionSeq](Diagrams/MissionExecutionSeq.md)
- **Trace requirements**: [RequirementTraceMermaid](Diagrams/RequirementTraceMermaid.md) → individual files under `Requirements/` and `Verification/`
- **Check decisions**: `Decisions/` for the rationale behind decomposition choices
